use ratatui::text::Span;
use tokio::sync::broadcast;

pub type LogLine = Vec<Span<'static>>;



lazy_static::lazy_static! {
    static ref LOG_TX: broadcast::Sender<LogLine> = broadcast::channel(500).0;
}

pub fn log(line: LogLine) {
    let _ = LOG_TX.send(line);
}

pub fn subscribe() -> broadcast::Receiver<LogLine> {
    LOG_TX.subscribe()
}