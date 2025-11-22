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
// TODO: Next, work on a multi-threaded thing and a server. don't use tokio cause that's just async.
