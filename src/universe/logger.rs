use crossterm::style::Color;

#[derive(Debug, Clone)]
pub enum LogMessage {
    Info(String),
    UniverseLog { name: String, color: Color, message: String },
}
