use tokio::sync::broadcast;
use rgb::RGB8;

pub type LogEntry = (String, Option<rgb::RGB8>); // (message, optional color)

lazy_static::lazy_static! {
    static ref LOG_TX: broadcast::Sender<LogEntry> = {
        let (tx, _) = broadcast::channel(1000);
        tx
    };
}

pub fn log(msg: impl Into<String>, color: Option<rgb::RGB8>) {
    let _ = LOG_TX.send((msg.into(), color));
}

pub fn subscribe() -> broadcast::Receiver<LogEntry> {
    LOG_TX.subscribe()
}