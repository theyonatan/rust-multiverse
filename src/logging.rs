use tokio::sync::broadcast;

pub type LogLine = String;

lazy_static::lazy_static! {
    static ref LOG_TX: broadcast::Sender<LogLine> = broadcast::channel(500).0;
}

pub fn log(line: impl Into<String>) {
    let _ = LOG_TX.send(line.into());
}

pub fn subscribe() -> broadcast::Receiver<LogLine> {
    LOG_TX.subscribe()
}