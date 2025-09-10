use std::{
    sync::{Arc, atomic::AtomicU64},
    time::Instant,
};

use crate::event::UploadState;

#[derive(Debug, Default)]
pub struct Upload {
    start: Option<Instant>,
    end: Option<Instant>,
    uploaded: Arc<AtomicU64>,
}

impl Upload {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn clone_uploaded(&self) -> Arc<AtomicU64> {
        self.uploaded.clone()
    }
}

impl Upload {
    pub fn apply_state(&mut self, state: UploadState) {
        match state {
            UploadState::Start => self.start = Some(Instant::now()),
            UploadState::Done => self.end = Some(Instant::now()),
        }
    }
}
