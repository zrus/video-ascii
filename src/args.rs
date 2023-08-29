use ::std::path::PathBuf;

#[derive(Debug, ::clap::Parser)]
pub struct Args {
  /// Loop the video
  #[clap(short = 'l', long = "loop")]
  pub is_loop: bool,
  #[command(flatten)]
  pub input: Input,
}

#[derive(Debug, ::clap::Args)]
#[group(required = true, multiple = false)]
pub struct Input {
  /// From file
  #[arg(short, long)]
  #[arg(value_parser = is_file)]
  pub file: Option<String>,
  /// From link
  #[arg(short, long)]
  pub uri: Option<String>,
}

fn is_file(s: &str) -> Result<String, String> {
  let file = PathBuf::from(s);
  if !file.exists() {
    return Err(format!("Not found."));
  }
  if file.is_dir() {
    return Err(format!("Not a file."));
  }
  if file.is_file() {
    return Ok(s.to_owned());
  }
  return Err(format!("Invalid."));
}
