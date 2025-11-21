mod universe;

use universe::{create_universe, UniverseCommand};


fn main() {

    println!("Hello, world!");

    let universe_handle = create_universe();

    universe_handle.commander.send(UniverseCommand::Start).unwrap();
    universe_handle.commander.send(UniverseCommand::RequestState()).unwrap();
    std::thread::sleep(std::time::Duration::from_millis(4000));
    universe_handle.commander.send(UniverseCommand::Stop).unwrap();

    universe_handle.universe_thread.join().unwrap();
}


// TODO: Replace all .unwrap() statements with ? or a real handling case.
