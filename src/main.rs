use futures::StreamExt;
use gst::prelude::*;
use log::{debug, error, info, trace};
use std::os::fd::AsRawFd;

use crate::stream::ScreencastStream;

mod stream;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    env_logger::init();
    gst::init()?;

    let stream = ScreencastStream::start().await?;
    debug!("{:#?}", stream);

    let pipeline = create_pipeline(&stream)?;
    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let bus = pipeline.bus().unwrap();
    while let Some(msg) = bus.stream().next().await {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                break;
            }
            MessageView::Error(err) => {
                error!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            MessageView::StateChanged(change) => {
                info!("STATE: {:?}", change.current());
            }
            msg_view => {
                trace!("{:?}", msg_view);
            }
        };
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");

    Ok(())
}

/// make pipewiresrc -> videoconvert -> xvimagesink pipeline
fn create_pipeline(
    stream: &ScreencastStream,
) -> Result<gst::Pipeline, gst::glib::error::BoolError> {
    let pipeline = gst::Pipeline::default();

    let pipewiresrc = gst::ElementFactory::make("pipewiresrc")
        .property("fd", stream.pipe_wire_remote_fd.as_raw_fd())
        .property("always-copy", true)
        .property(
            "path",
            stream.pipe_wire_stream.pipe_wire_node_id().to_string(),
        )
        .build()?;
    let videoconvert = gst::ElementFactory::make("videoconvert").build()?;
    let xvimagesink = gst::ElementFactory::make("xvimagesink").build()?;

    pipeline.add_many([&pipewiresrc, &videoconvert, &xvimagesink])?;

    pipewiresrc.link(&videoconvert)?;
    videoconvert.link(&xvimagesink)?;

    Ok(pipeline)
}
