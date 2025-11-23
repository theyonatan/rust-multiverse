use std::io;
use tokio::io::AsyncBufReadExt;
use std::io::Write;

pub async fn menu_read_input() -> Result<String, io::Error> {
    print!("> ");
    // Flush stdout to ensure the prompt arrow appears before input on all terminals
    let _ = std::io::stdout().flush();

    let mut line = String::new();
    tokio::io::BufReader::new(tokio::io::stdin()).read_line(&mut line).await?;

    Ok(line.trim().to_string())
}

pub async fn menu_request_universe_name(action: &str) -> String {
    println!("\n--- [{} Universe] ---", action);
    println!("Enter universe name:");

    menu_read_input().await.unwrap_or_else(|_| String::new())
}

pub fn print_main_menu() {
    println!("\n================ MAIN MENU ================");
    println!("Commands:");
    println!("  [New]      -> Create a new universe");
    println!("  [Get]      -> List all universes");
    println!("  [Command]  -> Manage a specific universe");
    println!("  [Shutdown] -> Shutdown the supervisor");
    println!("-------------------------------------------");
}

pub fn print_command_menu(universe_name: &str) {
    println!("\n--- Managing Universe: '{}' ---", universe_name);
    println!("  [Start]    -> Resume universe execution");
    println!("  [Stop]     -> Pause universe execution");
    println!("  [Event]    -> Open the Event Injector menu");
    println!("  [State]    -> Request current state");
    println!("  [Shutdown] -> Destroy and delete this universe");
    println!("  [Back]     -> Return to Main Menu");
    println!("-------------------------------------------");
}

pub fn print_event_menu(universe_name: &str) {
    println!("\n--- Event Injection: '{}' ---", universe_name);
    println!("  [ChangeState] -> Input custom state string");
    println!("  [Shatter]     -> Damage the universe");
    println!("  [Crash]       -> Force crash");
    println!("  [Heal]        -> Heal universe");
    println!("  [Ping]        -> Send ping");
    println!("  [Pong]        -> Send pong");
    println!("  [Back]        -> Return to Command Menu");
    println!("-------------------------------------------");
}
