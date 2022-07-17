mod decoder;
mod reader;

use anyhow::Result;
use crossterm::{
  cursor::{Hide, Show},
  terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
  ExecutableCommand,
};
use reader::InputType;

const DENSITY: &'static str = "";

#[tokio::main]
async fn main() -> Result<()> {
  // let mut stdout = std::io::stdout();
  // terminal::enable_raw_mode()?;

  // stdout.execute(EnterAlternateScreen)?;
  // stdout.execute(Hide)?;

  // Main stuff goes here
  // ...
  // 1. Read video from file (later may support URI from Youtube and many else formats)
  let input = InputType::try_from("./data/test.mp4")?;
  println!("{input:?}");

  // stdout.execute(Show)?;
  // stdout.execute(LeaveAlternateScreen)?;
  // terminal::disable_raw_mode()?;

  Ok(())
}
