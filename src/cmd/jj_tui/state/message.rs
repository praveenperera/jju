use std::time::{Duration, Instant};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MessageKind {
    Success,
    Warning,
    Error,
}

pub struct StatusMessage {
    pub text: String,
    pub kind: MessageKind,
    pub expires: Instant,
}

impl StatusMessage {
    pub fn new(text: String, kind: MessageKind) -> Self {
        Self {
            text,
            kind,
            expires: Instant::now() + Duration::from_secs(3),
        }
    }

    pub fn with_duration(text: String, kind: MessageKind, duration: Duration) -> Self {
        Self {
            text,
            kind,
            expires: Instant::now() + duration,
        }
    }

    pub fn is_expired(&self) -> bool {
        Instant::now() > self.expires
    }
}
