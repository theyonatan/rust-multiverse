mod universe;
mod supervisor;
use supervisor::supervisor::UserSupervisor;

#[tokio::main]
async fn main() {
    println!("Hello, world!");

    let mut user_supervisor = UserSupervisor::new();

    user_supervisor.main_loop().await;
}


// TODO: Replace all .unwrap() statements with ? or a real handling case.
// TODO: Take care of logs in the vector and generally logs. maybe new thread for logs.
