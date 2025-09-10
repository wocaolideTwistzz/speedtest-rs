use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use crate::event::DownloadState;

#[derive(Debug, Default)]
pub struct Download {
    start: Option<Instant>,
    end: Option<Instant>,
    downloaded: Arc<AtomicU64>,
}

impl Download {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clone_downloaded(&self) -> Arc<AtomicU64> {
        self.downloaded.clone()
    }
}

impl Download {
    pub fn apply_state(&mut self, state: DownloadState) {
        match state {
            DownloadState::Start => self.start = Some(Instant::now()),
            DownloadState::Done => self.end = Some(Instant::now()),
        }
    }
}
