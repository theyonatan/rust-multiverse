mod universe;
mod supervisor;
mod terminal_ui;
use terminal_ui::TerminalUI;

use supervisor::supervisor::UserSupervisor;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut user_supervisor = UserSupervisor::new();
    let mut ui = TerminalUI::new(&mut user_supervisor);

    ui.run().await;
}


// TODO: Replace all .unwrap() statements with ? or a real handling case.
// TODO: Take care of logs in the vector and generally logs. maybe new thread for logs.
