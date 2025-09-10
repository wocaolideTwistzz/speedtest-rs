use std::time::Instant;

use speedtest_rs_core::model::Server;

use crate::event::FetchServersState;

#[derive(Debug, Default)]
pub struct FetchServers {
    start: Option<Instant>,
    end: Option<Instant>,
    result: Option<FetchServersResult>,
}

impl FetchServers {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn apply_state(&mut self, state: FetchServersState) {
        match state {
            FetchServersState::Start => self.start = Some(Instant::now()),
            FetchServersState::Success(servers) => {
                self.end = Some(Instant::now());
                self.result = Some(FetchServersResult::Success(servers));
            }
            FetchServersState::Failed(e) => {
                self.end = Some(Instant::now());
                self.result = Some(FetchServersResult::Error(e))
            }
        }
    }
}

#[derive(Debug)]
enum FetchServersResult {
    Success(Vec<Server>),
    Error(String),
}
