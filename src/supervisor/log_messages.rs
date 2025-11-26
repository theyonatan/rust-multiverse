use ratatui::text::Span;
use ratatui::style::{Color, Style};
use rgb::RGB8;
use crate::logging::log;

pub struct Log;

impl Log {
    fn color(rgb: RGB8) -> Color {
        Color::Rgb(rgb.r, rgb.g, rgb.b)
    }

    pub fn info(msg: impl Into<String>) {
        let msg = msg.into();
        let spans = vec![
            Span::styled("> ".to_owned(), Style::default().fg(Color::Cyan)),
            Span::raw(msg),
        ];
        log(spans);
    }

    pub fn created(name: &str, color: RGB8) {
        let spans = vec![
            Span::styled("> Created universe ".to_owned(), Style::default().fg(Color::Cyan)),
            Span::styled(name.to_owned(), Style::default().fg(Self::color(color))),
        ];
        log(spans);
    }

    pub fn attack(source: &str, source_color: RGB8, target: &str, target_color: RGB8, dmg: i32) {
        let spans = vec![
            Span::raw("[".to_owned()),
            Span::styled(source.to_owned(), Style::default().fg(Self::color(source_color))),
            Span::raw("] → [".to_owned()),
            Span::styled(target.to_owned(), Style::default().fg(Self::color(target_color))),
            Span::styled(format!("] −{dmg} HP"), Style::default().fg(Color::Red)),
        ];
        log(spans);
    }

    pub fn heal(source: &str, source_color: RGB8, target: &str, target_color: RGB8, amount: i32) {
        let spans = vec![
            Span::raw("[".to_owned()),
            Span::styled(source.to_owned(), Style::default().fg(Self::color(source_color))),
            Span::raw("] healed [".to_owned()),
            Span::styled(target.to_owned(), Style::default().fg(Self::color(target_color))),
            Span::styled(format!("] +{amount} HP"), Style::default().fg(Color::Green)),
        ];
        log(spans);
    }

    pub fn collapsed(name: &str, color: RGB8) {
        let spans = vec![
            Span::styled("☠ ".to_owned(), Style::default().fg(Color::Red)),
            Span::styled(name.to_owned(), Style::default().fg(Self::color(color))),
            Span::styled(" has COLLAPSED".to_owned(), Style::default().fg(Color::DarkGray)),
        ];
        log(spans);
    }

    pub fn user_action(actor: &str, action: &str, target: &str, color: RGB8) {
        let spans = vec![
            Span::styled("> ".to_owned(), Style::default().fg(Color::Cyan)),
            Span::styled(actor.to_owned(), Style::default().fg(Self::color(color))),
            Span::raw(format!(" {action} {target}")),
        ];
        log(spans);
    }
}