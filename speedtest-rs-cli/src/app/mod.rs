use std::{
    collections::VecDeque,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering},
    },
    time::Instant,
};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use speedtest_rs_core::{model::Server, speed_tester::SpeedTester};
use tokio::sync::mpsc;

use crate::{
    app::progress::Progress,
    event::{AppEvent, Event, EventHandler, State, Status},
};

pub mod progress;

const MAX_RECORDS_LEN: usize = 20;

const RECORD_INTERVAL_SECS: f32 = 0.5;

#[derive(Debug)]
pub struct App {
    pub running: bool,

    pub events: EventHandler,

    pub fetch_config: Progress<SimpleConfig>,

    pub fetch_servers: Progress<Vec<Server>>,

    pub racing_servers: Progress<Server>,

    pub download: Progress<()>,

    pub upload: Progress<()>,

    pub downloaded: Arc<AtomicU64>,

    pub uploaded: Arc<AtomicU64>,

    pub servers_scroll: usize,

    pub max_servers_scroll: usize,

    pub downloaded_data: VecDeque<u64>,

    pub uploaded_data: VecDeque<u64>,

    pub last_download_time: Option<Instant>,

    pub last_download_count: Option<u64>,

    pub last_upload_time: Option<Instant>,

    pub last_upload_count: Option<u64>,

    shutdown_tx: tokio::sync::watch::Sender<bool>,

    shutdown_rx: tokio::sync::watch::Receiver<bool>,

    speed_tester: SpeedTester,
}

impl Default for App {
    fn default() -> Self {
        let (shutdown_tx, shutdown_rx) = tokio::sync::watch::channel(false);
        Self {
            running: true,
            events: EventHandler::new(),
            fetch_config: Progress::new("Fetch Config"),
            fetch_servers: Progress::new("Fetch Servers"),
            racing_servers: Progress::new("Racing Servers"),
            download: Progress::new("Download"),
            upload: Progress::new("Upload"),

            downloaded: Arc::new(AtomicU64::new(0)),
            uploaded: Arc::new(AtomicU64::new(0)),

            servers_scroll: 0,
            max_servers_scroll: 0,

            downloaded_data: VecDeque::with_capacity(MAX_RECORDS_LEN),
            uploaded_data: VecDeque::with_capacity(MAX_RECORDS_LEN),
            last_download_time: None,
            last_upload_time: None,
            last_download_count: None,
            last_upload_count: None,

            shutdown_tx,
            shutdown_rx,
            speed_tester: SpeedTester::default(),
        }
    }
}

impl App {
    pub fn new() -> Self {
        Self::default()
    }

    pub async fn run(mut self, mut terminal: DefaultTerminal) -> color_eyre::Result<()> {
        self.spawn_speed_test();

        while self.running {
            terminal.draw(|frame| frame.render_widget(&self, frame.area()))?;

            match self.events.next().await? {
                Event::Tick => self.tick(),
                Event::Crossterm(event) => {
                    if let crossterm::event::Event::Key(key_event) = event {
                        self.handle_key_events(key_event)?;
                    }
                }
                Event::App(app_event) => self.handle_app_events(app_event)?,
            }
        }
        Ok(())
    }

    pub fn handle_key_events(&mut self, key_event: KeyEvent) -> color_eyre::Result<()> {
        match key_event.code {
            KeyCode::Esc | KeyCode::Char('q') => self.events.send(AppEvent::Quit),
            KeyCode::Char('c' | 'C' | 'd' | 'D')
                if key_event.modifiers == KeyModifiers::CONTROL =>
            {
                self.events.send(AppEvent::Quit);
            }
            KeyCode::Char('j') | KeyCode::Down => {
                self.scroll_down();
            }
            KeyCode::Char('k') | KeyCode::Up => {
                self.scroll_up();
            }
            _ => (),
        }
        Ok(())
    }

    pub fn handle_app_events(&mut self, app_event: AppEvent) -> color_eyre::Result<()> {
        match app_event {
            AppEvent::Quit => self.quit(),
            AppEvent::SetState(state) => {
                let should_cancel = if state.is_error() {
                    Some(state.cancel_after())
                } else {
                    None
                };
                match state {
                    State::FetchConfig(st) => self.fetch_config.apply_status(st),
                    State::FetchServers(st) => {
                        if let Status::Ok(servers) = &st {
                            self.max_servers_scroll = servers.len();
                        }
                        self.fetch_servers.apply_status(st);
                    }
                    State::RacingServers(st) => self.racing_servers.apply_status(st),
                    State::Download(st) => {
                        match &st {
                            Status::Start => self.last_download_time = Some(Instant::now()),
                            Status::Ok(_) | Status::Err(_) => {
                                let downloaded = self.downloaded.load(Ordering::SeqCst)
                                    - self.last_download_count.unwrap_or(0);
                                let elapsed =
                                    self.last_download_time.unwrap().elapsed().as_secs_f32();

                                self.downloaded_data
                                    .push_back((downloaded as f32 / elapsed) as u64);
                                self.last_download_count = Some(downloaded);
                            }
                            _ => {}
                        }
                        self.download.apply_status(st)
                    }
                    State::Upload(st) => {
                        match &st {
                            Status::Start => self.last_upload_time = Some(Instant::now()),
                            Status::Ok(_) | Status::Err(_) => {
                                let uploaded = self.uploaded.load(Ordering::SeqCst)
                                    - self.last_upload_count.unwrap_or(0);
                                let elapsed =
                                    self.last_upload_time.unwrap().elapsed().as_secs_f32();

                                self.uploaded_data
                                    .push_back((uploaded as f32 / elapsed) as u64);
                                self.last_upload_count = Some(uploaded);
                            }
                            _ => {}
                        }
                        self.upload.apply_status(st)
                    }
                };

                if let Some(cancel_list) = should_cancel {
                    for state in cancel_list {
                        self.events.send(AppEvent::SetState(state));
                    }
                }
            }
        }
        Ok(())
    }

    pub fn scroll_down(&mut self) {
        self.servers_scroll = self
            .servers_scroll
            .saturating_add(1)
            .min(self.max_servers_scroll);
    }

    pub fn scroll_up(&mut self) {
        self.servers_scroll = self.servers_scroll.saturating_sub(1);
    }

    pub fn tick(&mut self) {
        if let Some(start) = self.last_download_time
            && let Status::Start = self.download.status()
        {
            let now = Instant::now();
            let elapsed = now.duration_since(start).as_secs_f32();

            if elapsed >= RECORD_INTERVAL_SECS {
                let current_downloaded = self.downloaded.load(Ordering::SeqCst);

                let speed =
                    (current_downloaded - self.last_download_count.unwrap_or(0)) as f32 / elapsed;

                self.downloaded_data.push_back(speed as u64);
                self.last_download_count = Some(current_downloaded);
                self.last_download_time = Some(now);
            }
        }

        if let Some(start) = self.last_upload_time
            && let Status::Start = self.upload.status()
        {
            let now = Instant::now();
            let elapsed = now.duration_since(start).as_secs_f32();

            if elapsed >= RECORD_INTERVAL_SECS {
                let current_uploaded = self.uploaded.load(Ordering::SeqCst);

                let speed =
                    (current_uploaded - self.last_upload_count.unwrap_or(0)) as f32 / elapsed;

                self.uploaded_data.push_back(speed as u64);
                self.last_upload_count = Some(current_uploaded);
                self.last_upload_time = Some(now);
            }
        }
    }

    pub fn quit(&mut self) {
        _ = self.shutdown_tx.send(true);
        self.running = false
    }

    fn spawn_speed_test(&self) {
        let speed_tester = self.speed_tester.clone();
        let sender = self.events.clone_sender();
        let downloaded = self.downloaded.clone();
        let uploaded = self.uploaded.clone();
        let mut shutdown = self.shutdown_rx.clone();

        tokio::spawn(async move {
            tokio::select! {
                biased;
                _ = shutdown.changed() => {},
                _ = App::speedtest(speed_tester, sender, downloaded, uploaded) => {}
            };
        });
    }

    pub async fn speedtest(
        speed_tester: SpeedTester,
        sender: mpsc::UnboundedSender<Event>,
        downloaded: Arc<AtomicU64>,
        uploaded: Arc<AtomicU64>,
    ) {
        _ = sender.send(State::FetchConfig(Status::Start).into());

        let config = match speed_tester.fetch_config().await {
            Ok(config) => {
                _ = sender.send(State::FetchConfig(Status::Ok((&config).into())).into());
                config
            }
            Err(e) => {
                _ = sender.send(State::FetchConfig(Status::Err(e.to_string())).into());
                return;
            }
        };

        _ = sender.send(State::FetchServers(Status::Start).into());

        let servers = match speed_tester.fetch_servers(config.threads()).await {
            Ok(servers) => {
                _ = sender
                    .send(State::FetchServers(Status::Ok(servers.servers.servers.clone())).into());
                servers
            }
            Err(e) => {
                _ = sender.send(State::FetchServers(Status::Err(e.to_string())).into());
                return;
            }
        };

        _ = sender.send(State::RacingServers(Status::Start).into());
        let server = match speed_tester
            .select_fastest_server(servers.servers.servers)
            .await
        {
            Ok(server) => {
                _ = sender.send(State::RacingServers(Status::Ok(server.clone())).into());
                server
            }
            Err(e) => {
                _ = sender.send(State::RacingServers(Status::Err(e.to_string())).into());
                return;
            }
        };

        _ = sender.send(State::Download(Status::Start).into());
        speed_tester.download(&config, &server, downloaded).await;
        _ = sender.send(State::Download(Status::Ok(())).into());

        _ = sender.send(State::Upload(Status::Start).into());
        speed_tester.upload(&config, &server, uploaded).await;
        _ = sender.send(State::Upload(Status::Ok(())).into());
    }

    pub fn max_download_byte_ps(&self) -> usize {
        *self.downloaded_data.iter().max().unwrap_or(&0) as usize
    }

    pub fn min_download_byte_ps(&self) -> usize {
        *self.downloaded_data.iter().min().unwrap_or(&0) as usize
    }

    pub fn latest_download_byte_ps(&self) -> usize {
        *self.downloaded_data.iter().last().unwrap_or(&0) as usize
    }

    pub fn total_download_bytes(&self) -> usize {
        self.downloaded.load(Ordering::SeqCst) as usize
    }

    pub fn avg_download_byte_ps(&self) -> usize {
        (self.downloaded_data.iter().sum::<u64>() as usize) / self.downloaded_data.len().max(1)
    }

    pub fn max_upload_byte_ps(&self) -> usize {
        *self.uploaded_data.iter().max().unwrap_or(&0) as usize
    }

    pub fn min_upload_byte_ps(&self) -> usize {
        *self.uploaded_data.iter().min().unwrap_or(&0) as usize
    }

    pub fn avg_upload_byte_ps(&self) -> usize {
        (self.uploaded_data.iter().sum::<u64>() as usize) / self.uploaded_data.len().max(1)
    }

    pub fn latest_upload_byte_ps(&self) -> usize {
        *self.uploaded_data.iter().last().unwrap_or(&0) as usize
    }

    pub fn total_upload_bytes(&self) -> usize {
        self.uploaded.load(Ordering::SeqCst) as usize
    }
}

#[derive(Debug, Clone)]
pub struct SimpleConfig {
    pub ip: String,
    pub latitude: String,
    pub longitude: String,
    pub isp: String,
    pub country: String,
}

impl From<&speedtest_rs_core::model::Config> for SimpleConfig {
    fn from(value: &speedtest_rs_core::model::Config) -> Self {
        Self {
            ip: value.client.ip.clone(),
            latitude: value.client.lat.to_string(),
            longitude: value.client.lon.to_string(),
            isp: value.client.isp.clone(),
            country: value.client.country.clone(),
        }
    }
}
