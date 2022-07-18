use std::io::{Stdout, Write};

use crossterm::{
  cursor::MoveTo,
  style::{Color, SetBackgroundColor},
  terminal::{Clear, ClearType},
  QueueableCommand,
};

use crate::frame::Frame;

pub fn render(stdout: &mut Stdout, last_frame: &Frame, curr_frame: &Frame, force: bool) {
  if force {
    stdout.queue(Clear(ClearType::All)).unwrap();
    stdout.queue(SetBackgroundColor(Color::Black)).unwrap();
  }

  for (y, row) in curr_frame.iter().enumerate() {
    for (x, s) in row.iter().enumerate() {
      if *s != last_frame[y][x] {
        stdout.queue(MoveTo(y as u16, x as u16)).unwrap();
        println!("{}", *s);
      }
    }
  }
  stdout.flush().unwrap();
}
