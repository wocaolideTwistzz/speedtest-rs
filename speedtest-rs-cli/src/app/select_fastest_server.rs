use std::time::Instant;

use speedtest_rs_core::model::Server;

use crate::event::SelectFastestServerState;

#[derive(Debug, Default)]
pub struct SelectFastestServer {
    start: Option<Instant>,
    end: Option<Instant>,
    result: Option<SelectFastestServerResult>,
}

impl SelectFastestServer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl SelectFastestServer {
    pub fn apply_state(&mut self, state: SelectFastestServerState) {
        match state {
            SelectFastestServerState::Start => self.start = Some(Instant::now()),
            SelectFastestServerState::Success(v) => {
                self.end = Some(Instant::now());
                self.result = Some(SelectFastestServerResult::Success(v));
            }
            SelectFastestServerState::Failed(v) => {
                self.end = Some(Instant::now());
                self.result = Some(SelectFastestServerResult::Error(v));
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct SimpleConfig {
    pub client_ip: String,
    pub latitude: String,
    pub longitude: String,
    pub isp: String,
}

impl From<&speedtest_rs_core::model::Config> for SimpleConfig {
    fn from(value: &speedtest_rs_core::model::Config) -> Self {
        Self {
            client_ip: value.client_info().ip.clone(),
            latitude: value.client_info().lat.to_string(),
            longitude: value.client_info().lon.to_string(),
            isp: value.client_info().isp.clone(),
        }
    }
}

#[derive(Debug)]
enum SelectFastestServerResult {
    Success(Server),
    Error(String),
}
