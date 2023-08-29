use ::anyhow::{anyhow, Result};
use ::gst::element_error;
use ::gst::prelude::*;
use ::std::sync::mpsc::Sender;

use crate::reader::InputType;

pub struct Decoder {}

impl Decoder {
  pub fn create_pipeline(
    input: InputType,
    width: u32,
    height: u32,
    render_tx: Sender<Vec<u8>>,
  ) -> Result<gst::Pipeline> {
    gst::init()?;

    let pipeline = match input {
      InputType::Uri { .. } => todo!(),
      InputType::File { .. } => gst::parse_launch(&format!(
        "{} ! decodebin ! videoconvert ! videorate ! video/x-raw,framerate=30/1 ! videoscale ! video/x-raw,width={width},height={height} ! appsink name=sink",
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

    appsink.set_property("sync", false);

    appsink.set_caps(Some(
      &gst::Caps::builder("video/x-raw")
        .field("format", gst_video::VideoFormat::Rgbx.to_str())
        .build(),
    ));

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

          let frame =
            gst_video::VideoFrameRef::from_buffer_ref_readable(buffer, &info).map_err(|_| {
              element_error!(
                appsink,
                gst::ResourceError::Failed,
                ("Failed to map buffer readable")
              );
              gst::FlowError::Error
            })?;

          _ = render_tx.send(frame.plane_data(0).unwrap().to_vec());

          Ok(gst::FlowSuccess::Ok)
        })
        .build(),
    );

    pipeline.set_state(gst::State::Playing)?;

    Ok(pipeline)
  }
}
