#![allow(warnings, unused)]

mod decoder;
mod frame;
mod reader;
mod render;
mod util;

use anyhow::{bail, Result};
use crossterm::{
  cursor::{Hide, Show},
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use terminal_size::{terminal_size, Height, Width};

use reader::InputType;
use tokio::sync::mpsc;

use crate::{
  decoder::{main_loop, Decoder},
  util::map,
};

const DENSITY: &'static str = "Ñ@#W$9876543210?!abc;:+=-,._                    ";

#[tokio::main]
async fn main() -> Result<()> {
  let input = InputType::try_from("./data/test.mp4")?;
  println!("{input:?}");

  let size = terminal_size();
  let (width, height) = if let Some((Width(w), Height(h))) = size {
    println!("{w} {h}");
    (w as usize, h as usize)
  } else {
    bail!("Unable to get current terminal size");
  };

  let mut stdout = std::io::stdout();
  terminal::enable_raw_mode()?;
  stdout.execute(EnterAlternateScreen)?;
  stdout.execute(Hide)?;

  let (render_tx, mut render_rx) = mpsc::channel::<Vec<u8>>(1);
  let render_handler = tokio::spawn(async move {
    let mut last_frame = frame::new_frame(width, height);
    let mut stdout = std::io::stdout();
    render::render(&mut stdout, &last_frame, &last_frame, true);
    let density_len = DENSITY.len() as u8;
    let density = DENSITY.chars().collect::<Vec<char>>();
    loop {
      let curr_frame = match render_rx.recv().await {
        Some(data) => {
          let mut new_frame = frame::new_frame(width, height);
          for y in 0..height {
            for x in 0..width {
              let pixel_idx = (x + y * width) * 4;
              let r = data[pixel_idx + 0];
              let g = data[pixel_idx + 1];
              let b = data[pixel_idx + 2];
              let avg = (r + g + b) / 3;
              // let avg = (min!(r, g, b) + max!(r, g, b)) / 2;
              let char_idx = map(avg, 0..=255, 0..=density_len);
              new_frame[y][x] = density[char_idx as usize].to_string();
            }
          }
          new_frame
        }
        None => continue,
      };
      render::render(&mut stdout, &last_frame, &curr_frame, false);
      last_frame = curr_frame;
    }
  });

  let pipeline = Decoder::create_pipeline(input, width, height, render_tx)?;
  let (stop_tx, mut stop_rx) = mpsc::channel::<()>(1);
  main_loop(pipeline, stop_tx).await;
  stop_rx.recv().await;

  render_handler.abort();
  stdout.execute(Show)?;
  stdout.execute(LeaveAlternateScreen)?;
  terminal::disable_raw_mode()?;

  Ok(())
}
