use tokio::sync::broadcast;
use serde::{Serialize, Deserialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum LogLevel {
    Info,
    Universe,
    Relationship,
    UserAction,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct LogEntry {
    pub level: LogLevel,
    pub message: String,
}

impl LogEntry {
    pub fn new(level: LogLevel, message: impl Into<String>) -> Self {
        Self {
            level,
            message: message.into(),
        }
    }
}

lazy_static::lazy_static! {
    static ref LOG_TX: broadcast::Sender<LogEntry> = broadcast::channel(500).0;
}

/// Send a log entry to all subscribers.
pub fn log(entry: LogEntry) {
    let _ = LOG_TX.send(entry);
}

/// Subscribe to the log stream (used by the web server to collect recent logs).
pub fn subscribe() -> broadcast::Receiver<LogEntry> {
    LOG_TX.subscribe()
}
