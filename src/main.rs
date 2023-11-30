use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};

use futures::StreamExt;
use gst::prelude::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let proxy = Screencast::new().await?;
    let session = proxy.create_session().await?;

    let multiple = false;
    let restore_token = None;
    let cursor_mode = CursorMode::Embedded;
    let source_type = SourceType::Monitor | SourceType::Window;
    let persist_mode = PersistMode::DoNot;

    proxy
        .select_sources(
            &session,
            cursor_mode,
            source_type,
            multiple,
            restore_token,
            persist_mode,
        )
        .await?;

    let response = proxy
        .start(&session, &WindowIdentifier::default())
        .await?
        .response()?;
    let stream = response.streams().iter().next().unwrap();
    let node = stream.pipe_wire_node_id();
    println!("node id: {}", node);

    let fd = proxy.open_pipe_wire_remote(&session).await?;
    println!("fd: {}", fd);

    gst::init()?;

    let desc = [
        format!("pipewiresrc fd={fd} path={node}"),
        format!("videoconvert"),
        format!("xvimagesink synchronous=false"),
    ]
    .join(" ! ");
    let pipeline: gst::Pipeline = gst::parse_launch(&desc)?.downcast().unwrap();

    pipeline
        .set_state(gst::State::Playing)
        .expect("Unable to set the pipeline to the `Playing` state");

    let bus = pipeline.bus().unwrap();

    while let Some(msg) = bus.stream().next().await {
        use gst::MessageView;

        match msg.view() {
            MessageView::Eos(..) => {
                println!("received eos");
                break;
            }
            MessageView::Error(err) => {
                println!(
                    "Error from {:?}: {} ({:?})",
                    err.src().map(|s| s.path_string()),
                    err.error(),
                    err.debug()
                );
                break;
            }
            _ => (),
        };
    }

    pipeline
        .set_state(gst::State::Null)
        .expect("Unable to set the pipeline to the `Null` state");

    Ok(())
}
