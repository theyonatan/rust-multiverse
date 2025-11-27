use rgb::RGB8;
use crate::logging::{log, LogEntry, LogLevel};

pub struct Log;

impl Log {
    fn color_hex(rgb: RGB8) -> String {
        format!("#{:02X}{:02X}{:02X}", rgb.r, rgb.g, rgb.b)
    }

    fn universe_label(name: &str, color: RGB8) -> String {
        // e.g. "Alpha[#FF00AA]"
        format!("{}[{}]", name, Self::color_hex(color))
    }

    pub fn info(msg: impl Into<String>) {
        let message = format!("> {}", msg.into());
        log(LogEntry::new(LogLevel::Info, message));
    }

    pub fn created(name: &str, color: RGB8) {
        let message = format!(
            "> Created universe {}",
            Self::universe_label(name, color)
        );
        log(LogEntry::new(LogLevel::Universe, message));
    }

    pub fn attack(
        source_name: &str,
        source_color: RGB8,
        target_name: &str,
        target_color: RGB8,
        damage: i32,
    ) {
        let message = format!(
            "⚔ {} dealt {} damage to {}",
            Self::universe_label(source_name, source_color),
            damage,
            Self::universe_label(target_name, target_color),
        );
        log(LogEntry::new(LogLevel::Universe, message));
    }

    pub fn heal(
        source_name: &str,
        source_color: RGB8,
        target_name: &str,
        target_color: RGB8,
        amount: i32,
    ) {
        let message = format!(
            "✚ {} healed {} by {}",
            Self::universe_label(source_name, source_color),
            Self::universe_label(target_name, target_color),
            amount,
        );
        log(LogEntry::new(LogLevel::Universe, message));
    }

    pub fn collapsed(name: &str, color: RGB8) {
        let message = format!(
            "☠ {} has COLLAPSED",
            Self::universe_label(name, color),
        );
        log(LogEntry::new(LogLevel::Universe, message));
    }

    #[allow(clippy::too_many_arguments)]
    pub fn relationship_announcement(
        target_name: &str,
        target_color: RGB8,
        own_name: &str,
        own_color: RGB8,
        relationship_label: &str,
        flavor_text: &str,
    ) {
        let message = format!(
            "☯ {} and {} are now {}. {}",
            Self::universe_label(target_name, target_color),
            Self::universe_label(own_name, own_color),
            relationship_label,
            flavor_text,
        );
        log(LogEntry::new(LogLevel::Relationship, message));
    }

    pub fn user_action(actor: &str, action: &str, target: &str, _color: RGB8) {
        // You can add color info later if your web UI wants it.
        let message = format!("> {actor} {action} {target}");
        log(LogEntry::new(LogLevel::UserAction, message));
    }
}
