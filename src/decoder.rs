use byte_slice_cast::AsSliceOf;
use gst::MessageView;
use gst::element_error;
use gst::prelude::*;

use anyhow::{anyhow, Result};

use crate::reader::InputType;

pub struct Decoder {}

impl Decoder {
  pub fn create_pipeline(input: InputType) -> Result<gst::Pipeline> {
    gst::init()?;

    let pipeline = match input {
      InputType::Uri { .. } => todo!(),
      InputType::File { .. } => gst::parse_launch(&format!(
        "{} ! decodebin ! videoconvert ! appsink name=sink",
        input.to_gst_src()?
      ))?
      .downcast::<gst::Pipeline>()
      .map_err(|_| anyhow!("Unable to downcast to pipeline"))?,
    };

    let appsink = pipeline
      .by_name("sink")
      .ok_or(anyhow!("No element name \"sink\" found"))?
      .downcast::<gst_app::AppSink>()
      .map_err(|_| anyhow!("Unable to downcast to appsink"))?;

    appsink.set_property("sync", false)?;

    appsink.set_caps(Some(
      &gst::Caps::builder("video/x-raw")
        .field("format", gst_video::VideoFormat::Rgbx.to_str())
        .build(),
    ));

    let mut got_snapshot = false;

    appsink.set_callbacks(
      gst_app::AppSinkCallbacks::builder()
      .new_sample(move |appsink| {
        let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
        let buffer = sample.buffer().ok_or_else(|| {
          element_error!(
            appsink,
            gst::ResourceError::Failed,
            ("Failed to get buffer from appsink")
          );
          gst::FlowError::Error
        })?;

        if got_snapshot {
          return Err(gst::FlowError::Eos);
        }
        got_snapshot = true;

        let caps = sample.caps().ok_or_else(|| {
          element_error!(
            appsink,
            gst::ResourceError::Failed,
            ("Failed to get caps from sample")
          );
          gst::FlowError::Error
        })?;
        let info = gst_video::VideoInfo::from_caps(caps).map_err(|_| {
          element_error!(
            appsink,
            gst::ResourceError::Failed,
            ("Failed to get caps from sample")
          );
          gst::FlowError::Error
        })?;

        let frame = gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info).map_err(|_| {
          element_error!(
            appsink,
            gst::ResourceError::Failed,
            ("Failed to map buffer readable")
          );
          gst::FlowError::Error
        })?;

        println!("{:?}", frame.plane_data(0).unwrap());

        Ok(gst::FlowSuccess::Ok)
      }).build()
    );

    Ok(pipeline)
  }
}

pub fn main_loop(pipeline: gst::Pipeline) -> Result<()> {
  pipeline.set_state(gst::State::Playing)?;

  let bus = pipeline.bus().ok_or(anyhow!("Unable to get bus from pipeline"))?;

  for msg in bus.iter_timed(gst::ClockTime::NONE) {
    if let MessageView::Eos(..) = msg.view() {
      break;
    }
  }

  pipeline.set_state(gst::State::Null)?;

  Ok(())
}