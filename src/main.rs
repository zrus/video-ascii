// #![allow(warnings, unused)]

mod decoder;
mod frame;
mod reader;
mod render;
mod util;

use anyhow::{bail, Result};
use crossterm::{
  cursor::{Hide, Show},
  event::{self, Event, KeyCode},
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use gst::traits::ElementExt;
use terminal_size::{terminal_size, Height, Width};

use reader::InputType;
use tokio::sync::mpsc;

use crate::{decoder::Decoder, util::map};

const DENSITY: &'static str = "        _.,-=+:;cba!?0123456789$W#@Ñ";

#[tokio::main]
async fn main() -> Result<()> {
  let input = InputType::try_from("./data/test.mp4")?;
  println!("{input:?}");

  let size = terminal_size();
  let (width, height) = if let Some((Width(w), Height(h))) = size {
    println!("{w}:{h}");
    // let h = h - 2;
    // let w = h * 4;
    // (w as usize, h as usize)
    (234, 56)
  } else {
    bail!("Unable to get current terminal size");
  };

  let mut stdout = std::io::stdout();
  terminal::enable_raw_mode()?;
  stdout.execute(EnterAlternateScreen)?;
  stdout.execute(Hide)?;

  let (render_tx, mut render_rx) = mpsc::channel::<Vec<u8>>(1);
  let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
  let render_handler = tokio::spawn(async move {
    let pipeline = Decoder::create_pipeline(input, width as u32, height as u32, render_tx).unwrap();

    let mut last_frame = frame::new_frame(width, height);
    let mut stdout = std::io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    let density = DENSITY.chars().collect::<Vec<char>>();
    let density_len = density.len() as u16;
    while let Some(data) = render_rx.recv().await {
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
      render::render(&mut stdout, &last_frame, &curr_frame, false);
      last_frame = curr_frame;
    }

    pipeline.set_state(gst::State::Null).unwrap();
    stop_tx.send(()).await.unwrap();
  });

  tokio::select! {
    _ = control() => {
      println!("done control");
    }
    _ = stop_rx.recv() => {
      println!("end video");
    }
  }

  stdout.execute(Show)?;
  stdout.execute(LeaveAlternateScreen)?;
  terminal::disable_raw_mode()?;

  render_handler.abort();

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
