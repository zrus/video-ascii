use ::std::path::PathBuf;

use ::anyhow::{anyhow, Result};

use crate::args::Args;

#[derive(Debug, Clone)]
pub enum InputType {
  Uri { uri: String },
  // LiveURI { uri: &'a str },
  // RtspSrc { uri: &'a str },
  // FtpSrc { uri: &'a str },
  File { path: String },
}

impl InputType {
  pub fn to_gst_src(&self) -> Result<String> {
    match self {
      InputType::Uri { uri } => Ok(format!("souphttpsrc location={uri}")),
      InputType::File { path } => Ok(format!("filesrc location={path}")),
      // _ => Err(anyhow!("Unable to create Gstreamer source from: {self:?}")),
    }
  }
}

impl From<&Args> for InputType {
  fn from(value: &Args) -> Self {
    match (value.input.file.as_deref(), value.input.uri.as_deref()) {
      (None, Some(uri)) => Self::Uri {
        uri: uri.to_owned(),
      },
      (Some(path), None) => Self::File {
        path: path.to_owned(),
      },
      _ => unreachable!(),
    }
  }
}

impl TryFrom<&str> for InputType {
  type Error = ::anyhow::Error;

  fn try_from(value: &str) -> Result<Self, Self::Error> {
    match value {
      _ if value.starts_with("https://") || value.starts_with("http://") => Ok(Self::Uri {
        uri: value.to_owned(),
      }),
      // TODO: handle live uri source.

      // _ if value.starts_with("rtsp://") => Ok(Self::RtspSrc { uri: value }),
      // _ if value.starts_with("ftp://") => Ok(Self::FtpSrc { uri: value }),
      _ if PathBuf::from(value).is_file() => Ok(Self::File {
        path: value.to_owned(),
      }),
      _ => Err(anyhow!("Unspecified input: {value}")),
    }
  }
}
