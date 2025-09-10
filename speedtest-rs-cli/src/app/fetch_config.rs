use std::time::{Duration, Instant};

use crate::event::FetchConfigState;

#[derive(Debug, Default)]
pub struct FetchConfig {
    start: Option<Instant>,
    end: Option<Instant>,
    result: Option<FetchConfigResult>,
}

impl FetchConfig {
    pub fn new() -> Self {
        Self::default()
    }
}

impl FetchConfig {
    pub fn apply_state(&mut self, state: FetchConfigState) {
        match state {
            FetchConfigState::Start => self.start = Some(Instant::now()),
            FetchConfigState::Success(v) => {
                self.end = Some(Instant::now());
                self.result = Some(FetchConfigResult::Success(v));
            }
            FetchConfigState::Failed(v) => {
                self.end = Some(Instant::now());
                self.result = Some(FetchConfigResult::Error(v));
            }
        }
    }

    pub fn is_start(&self) -> bool {
        self.start.is_some()
    }

    pub fn get_result(&self) -> Option<&FetchConfigResult> {
        self.result.as_ref()
    }

    pub fn elapsed(&self) -> Duration {
        let start = match self.start {
            Some(start) => start,
            None => return Duration::default(),
        };

        let end = match self.end {
            Some(end) => end,
            None => Instant::now(),
        };

        end.duration_since(start)
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

#[derive(Debug, Clone)]
pub enum FetchConfigResult {
    Success(SimpleConfig),
    Error(String),
}
