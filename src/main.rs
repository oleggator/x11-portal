use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType},
    WindowIdentifier,
};

use std::process::Command;

use gst::prelude::*;

//  gst-launch-1.0 -v videotestsrc pattern=snow ! autovideosink

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

    let output = Command::new("gst-launch-1.0")
        .arg("-v")
        .arg("pipewiresrc")
        .arg(format!("fd={}", fd))
        .arg(format!("path={}", node))
        // .arg("do-timestamp=true")
        // .arg("keepalive-time=1000")
        // .arg("resend-last=true")
        .arg("!")
        .arg("videoconvert")
        // .arg("!").arg("queue")
        .arg("!")
        .arg("xvimagesink")
        .arg("synchronous=false")
        .output()?;
    print!("{}", String::from_utf8(output.stdout)?);
    eprint!("{}", String::from_utf8(output.stderr)?);

    Ok(())
}
