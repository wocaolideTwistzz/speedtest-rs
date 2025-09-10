use std::sync::{Arc, atomic::AtomicU64};

use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::DefaultTerminal;
use speedtest_rs_core::speed_tester::SpeedTester;
use tokio::sync::mpsc;

use crate::{
    app::{
        download::Download, fetch_servers::FetchServers,
        select_fastest_server::SelectFastestServer, upload::Upload,
    },
    event::{AppEvent, Event, EventHandler, State},
};
use fetch_config::FetchConfig;

pub mod download;
pub mod fetch_config;
pub mod fetch_servers;
pub mod select_fastest_server;
pub mod upload;

#[derive(Debug)]
pub struct App {
    pub running: bool,

    pub events: EventHandler,

    pub fetch_config: FetchConfig,

    pub fetch_servers: FetchServers,

    pub select_fastest_server: SelectFastestServer,

    pub download: Download,

    pub upload: Upload,
}

impl Default for App {
    fn default() -> Self {
        Self {
            running: true,
            events: EventHandler::new(),
            fetch_config: FetchConfig::new(),
            fetch_servers: FetchServers::new(),
            select_fastest_server: SelectFastestServer::new(),
            download: Download::new(),
            upload: Upload::new(),
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
            _ => (),
        }
        Ok(())
    }

    pub fn handle_app_events(&mut self, app_event: AppEvent) -> color_eyre::Result<()> {
        match app_event {
            AppEvent::Quit => self.quit(),
            AppEvent::SetState(state) => {
                match state {
                    State::FetchConfig(st) => self.fetch_config.apply_state(st),
                    State::FetchServers(st) => self.fetch_servers.apply_state(st),
                    State::SelectFastestServer(st) => self.select_fastest_server.apply_state(st),
                    State::Download(st) => self.download.apply_state(st),
                    State::Upload(st) => self.upload.apply_state(st),
                };
            }
        }
        Ok(())
    }

    pub fn tick(&mut self) {}

    pub fn quit(&mut self) {
        self.running = false
    }

    fn spawn_speed_test(&self) {
        let sender = self.events.clone_sender();
        let downloaded = self.download.clone_downloaded();
        let uploaded = self.upload.clone_uploaded();

        tokio::spawn(App::speedtest(sender, downloaded, uploaded));
    }

    pub async fn speedtest(
        sender: mpsc::UnboundedSender<Event>,
        downloaded: Arc<AtomicU64>,
        uploaded: Arc<AtomicU64>,
    ) {
        let speed_tester = SpeedTester::default();

        _ = sender.send(AppEvent::start_fetch_config());

        let config = match speed_tester.fetch_config().await {
            Ok(config) => {
                _ = sender.send(AppEvent::fetch_config_success((&config).into()));
                config
            }
            Err(e) => {
                _ = sender.send(AppEvent::fetch_config_failed(e.to_string()));
                return;
            }
        };

        _ = sender.send(AppEvent::start_fetch_servers());

        let servers = match speed_tester.fetch_servers(config.threads()).await {
            Ok(servers) => {
                _ = sender.send(AppEvent::fetch_servers_success(
                    servers.servers.servers.clone(),
                ));
                servers
            }
            Err(e) => {
                _ = sender.send(AppEvent::fetch_config_failed(e.to_string()));
                return;
            }
        };

        _ = sender.send(AppEvent::start_select_fastest_server());
        let server = match speed_tester
            .select_fastest_server(servers.servers.servers)
            .await
        {
            Ok(server) => {
                _ = sender.send(AppEvent::select_fastest_server_success(server.clone()));
                server
            }
            Err(e) => {
                _ = sender.send(AppEvent::select_fastest_server_failed(e.to_string()));
                return;
            }
        };

        _ = sender.send(AppEvent::start_download());
        speed_tester.download(&config, &server, downloaded).await;
        _ = sender.send(AppEvent::download_done());

        _ = sender.send(AppEvent::start_upload());
        speed_tester.upload(&config, &server, uploaded).await;
        _ = sender.send(AppEvent::upload_done());
    }
}
