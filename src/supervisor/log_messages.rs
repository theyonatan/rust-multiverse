use rgb::RGB8;
use crate::logging::log;

pub struct Log;

impl Log {
    fn rgb(r: u8, g: u8, b: u8) -> String {
        format!("\x1b[38;2;{};{};{}m", r, g, b)
    }
    fn reset() -> &'static str { "\x1b[0m" }
    fn red() -> &'static str { "\x1b[31m" }
    fn green() -> &'static str { "\x1b[32m" }
    fn gray() -> &'static str { "\x1b[90m" }

    pub fn info(msg: impl Into<String>) {
        log(format!("> {}{}", msg.into(), Self::reset()));
    }

    pub fn attack(source: &str, source_color: RGB8, target: &str, target_color: RGB8, dmg: i32) {
        let s = Self::rgb(source_color.r, source_color.g, source_color.b);
        let t = Self::rgb(target_color.r, target_color.g, target_color.b);
        log(format!("[{s}{source}{} → {t}{target}{}] {}-{} HP{}", Self::reset(), Self::reset(), Self::red(), dmg, Self::reset()));
    }

    pub fn heal(source: &str, source_color: RGB8, target: &str, target_color: RGB8, amount: i32) {
        let s = Self::rgb(source_color.r, source_color.g, source_color.b);
        let t = Self::rgb(target_color.r, target_color.g, target_color.b);
        log(format!("[{s}{source}{}] healed [{t}{target}{}] {}+{} HP{}", Self::reset(), Self::reset(), Self::green(), amount, Self::reset()));
    }

    pub fn collapsed(name: &str, color: RGB8) {
        let c = Self::rgb(color.r, color.g, color.b);
        log(format!("☠ {c}{name}{} has COLLAPSED{}", Self::reset(), Self::gray()));
    }

    pub fn created(name: &str, color: RGB8) {
        let c = Self::rgb(color.r, color.g, color.b);
        log(format!("> Created universe {c}{name}{}", Self::reset()));
    }

    pub fn user_action(actor: &str, action: &str, target: &str, color: RGB8) {
        let c = Self::rgb(color.r, color.g, color.b);
        log(format!("> {c}{actor}{} {action} {target}", Self::reset()));
    }
}