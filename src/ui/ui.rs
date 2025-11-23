use ratatui::{
    backend::Backend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap},
    Frame,
};
use crate::ui::app::{App, AppState};
use crate::universe::logger::LogMessage;

pub fn ui<B: Backend>(f: &mut Frame, app: &App) {
    let chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([
            Constraint::Percentage(50), // Left: Commands
            Constraint::Percentage(50), // Right: Logs
        ].as_ref())
        .split(f.size());

    draw_left_panel::<B>(f, app, chunks[0]);
    draw_right_panel::<B>(f, app, chunks[1]);
}

fn draw_left_panel<B: Backend>(f: &mut Frame, app: &App, area: Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Help
            Constraint::Min(1),    // Menu/Status
            Constraint::Length(3), // Input
        ].as_ref())
        .split(area);

    // 1. Help Text
    let (msg, style) = {
        (
            vec![
                Span::raw("Press "),
                Span::styled("q", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to exit, "),
                Span::styled("e", Style::default().add_modifier(Modifier::BOLD)),
                Span::raw(" to enter commands."),
            ],
            Style::default().add_modifier(Modifier::RAPID_BLINK),
        )
    };
    let text = Text::from(Line::from(msg)).patch_style(style);
    let help_message = Paragraph::new(text)
        .block(Block::default().borders(Borders::ALL).title("Instructions"));
    f.render_widget(help_message, chunks[0]);

    // 2. Main Menu / Status Area
    let status_block = Block::default().borders(Borders::ALL).title("Status");
    let status_text = match &app.state {
        AppState::MainMenu => vec![
            Line::from(Span::styled("MAIN MENU", Style::default().fg(Color::Green).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from("Available Commands:"),
            Line::from("  new <name>       -> Create a new universe"),
            Line::from("  list             -> List all universes"),
            Line::from("  manage <name>    -> Control a specific universe"),
        ],
        AppState::ManagingUniverse(name) => vec![
            Line::from(vec![
                Span::raw("Managing: "),
                Span::styled(name, Style::default().fg(Color::Yellow).add_modifier(Modifier::BOLD)),
            ]),
            Line::from(""),
            Line::from("Available Commands:"),
            Line::from("  start / stop     -> Pause/Resume"),
            Line::from("  shatter / heal   -> Inject Event"),
            Line::from("  back             -> Return to Main Menu"),
        ],
    };
    let status_paragraph = Paragraph::new(status_text)
        .block(status_block)
        .wrap(Wrap { trim: true });
    f.render_widget(status_paragraph, chunks[1]);

    // 3. Input Box
    let input = Paragraph::new(app.input.as_str())
        .style({
            Style::default()
        })
        .block(Block::default().borders(Borders::ALL).title("Input"));
    f.render_widget(input, chunks[2]);
}

fn draw_right_panel<B: Backend>(f: &mut Frame, app: &App, area: Rect) {
    let messages: Vec<ListItem> = app.logs.iter().rev().map(|m| {
        let content = match m {
            LogMessage::Info(msg) => Line::from(Span::raw(msg)),
            LogMessage::UniverseLog { name, color, message } => {
                // Map Crossterm Color to Ratatui Color
                let r_color = match color {
                    crossterm::style::Color::Red => Color::Red,
                    crossterm::style::Color::Green => Color::Green,
                    crossterm::style::Color::Yellow => Color::Yellow,
                    crossterm::style::Color::Blue => Color::Blue,
                    crossterm::style::Color::Magenta => Color::Magenta,
                    crossterm::style::Color::Cyan => Color::Cyan,
                    _ => Color::White,
                };

                Line::from(vec![
                    Span::styled(format!("[{}] ", name), Style::default().fg(r_color).add_modifier(Modifier::BOLD)),
                    Span::raw(message)
                ])
            }
        };
        ListItem::new(content)
    }).collect();

    let messages_list = List::new(messages)
        .block(Block::default().borders(Borders::ALL).title("Event Log"))
        .direction(ratatui::widgets::ListDirection::BottomToTop);

    f.render_widget(messages_list, area);
}