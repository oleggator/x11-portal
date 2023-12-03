use ashpd::{
    desktop::screencast::{CursorMode, PersistMode, Screencast, SourceType, Stream},
    desktop::Session,
    WindowIdentifier,
};
use std::os::fd::{FromRawFd, OwnedFd};

#[derive(Debug)]
pub(super) struct ScreencastStream {
    _portal_session: Session<'static>,
    pub pipe_wire_stream: Stream,
    pub pipe_wire_remote_fd: OwnedFd,
}

impl ScreencastStream {
    pub(super) async fn start() -> Result<Self, ashpd::Error> {
        let proxy = Screencast::new().await?;
        let session = proxy.create_session().await?;

        let cursor_mode = CursorMode::Embedded;
        let source_type = SourceType::Monitor | SourceType::Window;
        let multiple = false;
        let restore_token = None;
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

        let pipe_wire_stream = response.streams().iter().next().unwrap().clone();
        let pipe_wire_remote_fd_raw = proxy.open_pipe_wire_remote(&session).await?;
        let pipe_wire_remote_fd = unsafe { OwnedFd::from_raw_fd(pipe_wire_remote_fd_raw) };

        Ok(Self {
            _portal_session: session,
            pipe_wire_stream,
            pipe_wire_remote_fd,
        })
    }
}
