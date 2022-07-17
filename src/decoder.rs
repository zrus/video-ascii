use anyhow::bail;
use gst::element_error;
use gst::prelude::*;

use anyhow::{Result, anyhow};

use crate::reader::InputType;

struct Decoder {}

impl Decoder {
  pub fn create_pipeline(input: InputType) -> Result<gst::Pipeline> {
    gst::init()?;

    let pipeline =
      gst::parse_launch(&format!("{}", input.to_gst_src()?))?.downcast::<gst::Pipeline>().map_err(|_| anyhow!("Unable to downcast to pipeline"))?;
    
    Ok(pipeline)
  }
}
