mod args;
mod decoder;
mod frame;
mod reader;
mod render;
mod util;

use ::std::{
  io::Stdout,
  sync::mpsc,
  time::{Duration, Instant},
};

use ::anyhow::{bail, Result};
use ::clap::Parser;
use ::crossterm::{
  cursor::{Hide, Show},
  event::{self, Event, KeyCode},
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use ::gst::traits::ElementExt;
use ::terminal_size::{terminal_size, Height, Width};

use reader::InputType;

use crate::{args::Args, decoder::Decoder, util::map};

const DENSITY: &'static str = "        _.,-=+:;cba!?0123456789$W#@Ñ";

#[tokio::main]
async fn main() -> Result<()> {
  let args = Args::parse();
  let is_loop = args.is_loop;
  let input = InputType::from(&args);

  let size = terminal_size();
  let (width, height) = if let Some((Width(w), Height(h))) = size {
    (w as usize, (h - 1) as usize)
  } else {
    bail!("Unable to get current terminal size");
  };

  let mut stdout = std::io::stdout();
  terminal::enable_raw_mode()?;
  stdout.execute(EnterAlternateScreen)?;
  stdout.execute(Hide)?;

  let (render_tx, render_rx) = mpsc::channel::<Vec<u8>>();
  let render_handler = tokio::spawn(async move {
    let pipeline = Decoder::create_pipeline(input, width as u32, height as u32, render_tx).unwrap();
    let mut frames = Vec::new();
    while let Ok(data) = render_rx.recv_timeout(Duration::from_millis(200)) {
      frames.push(data);
    }
    pipeline.set_state(gst::State::Null).unwrap();

    let mut last_frame = frame::new_frame(width, height);
    let mut stdout = std::io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    let density = DENSITY.chars().collect::<Vec<char>>();
    let density_len = density.len() as u16;
    let mut instant = Instant::now();

    while {
      for fr in &frames {
        render(
          &fr,
          width,
          height,
          &mut last_frame,
          &mut instant,
          &mut stdout,
          density_len,
          &density,
        )
        .await;
      }
      is_loop
    } {}
  });

  tokio::select! {
    _ = control() => {
      println!("done control");
    }
    _ = render_handler => {
      println!("end video");
    }
  }

  stdout.execute(Show)?;
  stdout.execute(LeaveAlternateScreen)?;
  terminal::disable_raw_mode()?;

  Ok(())
}

async fn control() {
  'mainloop: loop {
    while event::poll(std::time::Duration::default()).unwrap() {
      if let Event::Key(key_event) = event::read().unwrap() {
        match key_event.code {
          KeyCode::Esc | KeyCode::Char('q') => {
            println!("key hit!");
            break 'mainloop;
          }
          _ => {}
        }
      }
    }
    tokio::time::sleep(tokio::time::Duration::from_millis(1)).await;
  }
}

async fn render(
  data: &[u8],
  width: usize,
  height: usize,
  last_frame: &mut Vec<Vec<String>>,
  instant: &mut Instant,
  stdout: &mut Stdout,
  density_len: u16,
  density: &[char],
) {
  *instant = Instant::now();
  let mut curr_frame = frame::new_frame(width, height);
  for x in 0..width {
    for y in 0..height {
      let pixel_idx = (x + y * width) * 4;
      let r = data[pixel_idx + 0] as u16;
      let g = data[pixel_idx + 1] as u16;
      let b = data[pixel_idx + 2] as u16;
      let avg = (min!(r, g, b) + max!(r, g, b)) / 2;
      let char_idx = map(avg as u16, 0..256, 0..density_len);
      curr_frame[x][y] = density[char_idx as usize].to_string();
    }
  }
  render::render(stdout, &last_frame, &curr_frame, false);
  *last_frame = curr_frame;
  let delta = instant.elapsed();
  tokio::time::sleep(tokio::time::Duration::from_millis(std::cmp::max(
    33i64 - delta.as_millis() as i64,
    0i64,
  ) as u64))
  .await;
}
