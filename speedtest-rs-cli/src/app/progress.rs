use std::time::{Duration, Instant};

use crate::event::Status;

#[derive(Debug)]
pub struct Progress<T> {
    name: &'static str,
    start: Option<Instant>,
    end: Option<Instant>,
    status: Status<T>,
}

impl<T> Progress<T> {
    pub fn new(name: &'static str) -> Self {
        Self {
            name,
            start: None,
            end: None,
            status: Status::Pending,
        }
    }

    pub fn name(&self) -> &'static str {
        self.name
    }

    pub fn apply_status(&mut self, status: Status<T>) {
        if matches!(status, Status::Start) {
            self.start = Some(Instant::now());
        }
        if matches!(status, Status::Ok(_)) || matches!(status, Status::Err(_)) {
            self.end = Some(Instant::now());
        }
        self.status = status;
    }

    pub fn status(&self) -> &Status<T> {
        &self.status
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
