use color_eyre::eyre::OptionExt;
use crossterm::event::Event as CrosstermEvent;
use futures::{FutureExt, StreamExt};
use speedtest_rs_core::model::Server;
use std::time::Duration;
use tokio::sync::mpsc;

use crate::app::fetch_config::SimpleConfig;

/// The frequency at which tick events are emitted.
const TICK_FPS: f64 = 30.0;

/// Representation of all possible events.
#[derive(Clone, Debug)]
pub enum Event {
    /// An event that is emitted on a regular schedule.
    ///
    /// Use this event to run any code which has to run outside of being a direct response to a user
    /// event. e.g. polling external systems, updating animations, or rendering the UI based on a
    /// fixed frame rate.
    Tick,
    /// Crossterm events.
    ///
    /// These events are emitted by the terminal.
    Crossterm(CrosstermEvent),
    /// Application events.
    ///
    /// Use this event to emit custom events that are specific to your application.
    App(AppEvent),
}

/// Application events.
///
/// You can extend this enum with your own custom events.
#[derive(Clone, Debug)]
pub enum AppEvent {
    /// Quit the application.
    Quit,

    SetState(State),
}

/// Application state.
#[derive(Clone, Debug)]
pub enum State {
    /// Step1. Fetch config
    FetchConfig(FetchConfigState),

    /// Step2. Fetch servers
    FetchServers(FetchServersState),

    /// Step3. Select fastest server
    SelectFastestServer(SelectFastestServerState),

    /// Step4. Download
    Download(DownloadState),

    /// Step5. Upload
    Upload(UploadState),
}

#[derive(Clone, Debug)]
pub enum FetchConfigState {
    Start,
    Success(SimpleConfig),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum FetchServersState {
    Start,
    Success(Vec<Server>),
    Failed(String),
}

#[derive(Clone, Debug)]
pub enum SelectFastestServerState {
    Start,
    Success(Server),
    Failed(String),
}

#[derive(Debug, Clone)]
pub enum DownloadState {
    Start,
    Done,
}

#[derive(Debug, Clone)]
pub enum UploadState {
    Start,
    Done,
}

/// Terminal event handler.
#[derive(Debug)]
pub struct EventHandler {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
    /// Event receiver channel.
    receiver: mpsc::UnboundedReceiver<Event>,
}

impl Default for EventHandler {
    fn default() -> Self {
        Self::new()
    }
}

impl EventHandler {
    /// Constructs a new instance of [`EventHandler`] and spawns a new thread to handle events.
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        let actor = EventTask::new(sender.clone());
        tokio::spawn(async { actor.run().await });
        Self { sender, receiver }
    }

    /// Receives an event from the sender.
    ///
    /// This function blocks until an event is received.
    ///
    /// # Errors
    ///
    /// This function returns an error if the sender channel is disconnected. This can happen if an
    /// error occurs in the event thread. In practice, this should not happen unless there is a
    /// problem with the underlying terminal.
    pub async fn next(&mut self) -> color_eyre::Result<Event> {
        self.receiver
            .recv()
            .await
            .ok_or_eyre("Failed to receive event")
    }

    /// Queue an app event to be sent to the event receiver.
    ///
    /// This is useful for sending events to the event handler which will be processed by the next
    /// iteration of the application's event loop.
    pub fn send(&mut self, app_event: AppEvent) {
        // Ignore the result as the reciever cannot be dropped while this struct still has a
        // reference to it
        let _ = self.sender.send(Event::App(app_event));
    }

    pub fn clone_sender(&self) -> mpsc::UnboundedSender<Event> {
        self.sender.clone()
    }
}

/// A thread that handles reading crossterm events and emitting tick events on a regular schedule.
struct EventTask {
    /// Event sender channel.
    sender: mpsc::UnboundedSender<Event>,
}

impl EventTask {
    /// Constructs a new instance of [`EventThread`].
    fn new(sender: mpsc::UnboundedSender<Event>) -> Self {
        Self { sender }
    }

    /// Runs the event thread.
    ///
    /// This function emits tick events at a fixed rate and polls for crossterm events in between.
    async fn run(self) -> color_eyre::Result<()> {
        let tick_rate = Duration::from_secs_f64(1.0 / TICK_FPS);
        let mut reader = crossterm::event::EventStream::new();
        let mut tick = tokio::time::interval(tick_rate);

        loop {
            let tick_delay = tick.tick();
            let crossterm_event = reader.next().fuse();
            tokio::select! {
                _ = self.sender.closed() => {
                    break;
                }
                _ = tick_delay => {
                    self.send(Event::Tick);
                }
                Some(Ok(evt)) = crossterm_event => {
                    self.send(Event::Crossterm(evt));
                }
            };
        }
        Ok(())
    }

    /// Sends an event to the receiver.
    fn send(&self, event: Event) {
        // Ignores the result because shutting down the app drops the receiver, which causes the send
        // operation to fail. This is expected behavior and should not panic.
        let _ = self.sender.send(event);
    }
}

impl State {
    pub fn is_error(&self) -> bool {
        matches!(
            self,
            State::FetchConfig(FetchConfigState::Failed(_))
                | State::FetchServers(FetchServersState::Failed(_))
                | State::SelectFastestServer(SelectFastestServerState::Failed(_))
        )
    }

    pub fn is_done(&self) -> bool {
        false
    }
}

impl AppEvent {
    pub fn start_fetch_config() -> Event {
        Self::SetState(State::FetchConfig(FetchConfigState::Start)).into()
    }

    pub fn fetch_config_success(config: SimpleConfig) -> Event {
        Self::SetState(State::FetchConfig(FetchConfigState::Success(config))).into()
    }

    pub fn fetch_config_failed(error: String) -> Event {
        Self::SetState(State::FetchConfig(FetchConfigState::Failed(error))).into()
    }

    pub fn start_fetch_servers() -> Event {
        Self::SetState(State::FetchServers(FetchServersState::Start)).into()
    }

    pub fn fetch_servers_success(servers: Vec<Server>) -> Event {
        Self::SetState(State::FetchServers(FetchServersState::Success(servers))).into()
    }

    pub fn fetch_servers_failed(error: String) -> Event {
        Self::SetState(State::FetchServers(FetchServersState::Failed(error))).into()
    }

    pub fn start_select_fastest_server() -> Event {
        Self::SetState(State::SelectFastestServer(SelectFastestServerState::Start)).into()
    }

    pub fn select_fastest_server_success(server: Server) -> Event {
        Self::SetState(State::SelectFastestServer(
            SelectFastestServerState::Success(server),
        ))
        .into()
    }

    pub fn select_fastest_server_failed(error: String) -> Event {
        Self::SetState(State::SelectFastestServer(
            SelectFastestServerState::Failed(error),
        ))
        .into()
    }

    pub fn start_download() -> Event {
        Self::SetState(State::Download(DownloadState::Start)).into()
    }

    pub fn download_done() -> Event {
        Self::SetState(State::Download(DownloadState::Done)).into()
    }

    pub fn start_upload() -> Event {
        Self::SetState(State::Upload(UploadState::Start)).into()
    }

    pub fn upload_done() -> Event {
        Self::SetState(State::Upload(UploadState::Done)).into()
    }
}

impl From<AppEvent> for Event {
    fn from(app_event: AppEvent) -> Self {
        Event::App(app_event)
    }
}
