use std::path::PathBuf;

use anyhow::{anyhow, Result};

#[derive(Debug)]
pub enum InputType<'a> {
  Uri { uri: &'a str },
  // LiveURI { uri: &'a str },
  // RtspSrc { uri: &'a str },
  // FtpSrc { uri: &'a str },
  File { path: &'a str },
}

impl InputType<'_> {
  pub fn to_gst_src(&self) -> Result<String> {
    match self {
      InputType::Uri { uri } => Ok(format!("souphttpsrc location={uri}")),
      InputType::File { path } => Ok(format!("filesrc location={path}")),
      _ => Err(anyhow!("Unable to create Gstreamer source from: {self:?}")),
    }
  }
}

impl<'a> TryFrom<&'a str> for InputType<'a> {
  type Error = anyhow::Error;

  fn try_from(value: &'a str) -> Result<Self, Self::Error> {
    match value {
      _ if value.starts_with("https://") || value.starts_with("http://") => {
        Ok(Self::Uri { uri: value })
      }
      // TODO: handle live uri source.

      // _ if value.starts_with("rtsp://") => Ok(Self::RtspSrc { uri: value }),
      // _ if value.starts_with("ftp://") => Ok(Self::FtpSrc { uri: value }),
      _ if PathBuf::from(value).is_file() => Ok(Self::File { path: value }),
      _ => Err(anyhow!("Unspecified input: {value}")),
    }
  }
}
