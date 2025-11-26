mod universe;
mod supervisor;
mod terminal_ui;
mod logging;

use terminal_ui::TerminalUI;

use supervisor::user_supervisor::UserSupervisor;

#[tokio::main]
async fn main() {
    let mut user_supervisor = UserSupervisor::new();
    let mut ui = TerminalUI::new(&mut user_supervisor);

    ui.run().await;
}


// TODO: Replace all .unwrap() statements with ? or a real handling case.
// todo: check if I should replace all .clone with .to_owned()