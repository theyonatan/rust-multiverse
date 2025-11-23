mod universe;
mod supervisor;
mod ui;

use crate::ui::app::App;

#[tokio::main]
async fn main() {
    let mut app = App::new();
    
    app.run().await
}