use tokio::sync::mpsc::Sender;
use crate::universe::id::UniverseId;
use crate::universe::universe::Universe;
use crate::universe::universe_command::UniverseCommand;
use crate::universe::universe_event::UniverseEvent;

pub struct UniverseHandle {
    pub(crate) handle_id: UniverseId,
    pub(crate) commander_tx: Sender<UniverseCommand>,
    pub(crate) universe_task_handle: tokio::task::JoinHandle<()>,
}

impl UniverseHandle {
    fn new(mut universe: Universe) -> UniverseHandle {
        let handle_id = universe.id.clone();

        let (commander_tx, mut command_rx) = tokio::sync::mpsc::channel::<UniverseCommand>(10);
        let (event_tx, mut event_rx) = tokio::sync::mpsc::unbounded_channel::<UniverseEvent>();

        let universe_task_handle = tokio::spawn(async move{
            // todo: move to logger: println!("Spawned universe async instance!");

            loop {
                tokio::select! {
                    Some(command) = command_rx.recv() => {
                        // todo: move to logger: println!("Given universe command: {:?}", command);
                        match command {
                            UniverseCommand::Start => {
                                // todo - universe starts doing .step only if universe.activated
                                // this starts the universe
                            }
                            UniverseCommand::Stop => {
                                // todo - this just does universe.activated = false
                            }
                            UniverseCommand::InjectEvent(event) => {
                                // todo, new function to run universe event each step and this changes the event
                                // or pushes event to an event queue
                                // default event after event is finished should be somehting like "default"
                                // or "TickEvent".
                            }
                            UniverseCommand::RequestState() => {
                                // todo: logger thread
                                // should push a message to a queue that goes to the output stream
                                // or log, which a logger then logs on a different thread.
                                // good usage for threads here, use a logger, this pushes to logger,
                                // on a different thread a logger exists and prints all from queue.
                            }
                            UniverseCommand::Shutdown => {
    tokio::time::sleep(std::time::Duration::from_millis(4000)).await;

                                return;
                                // todo:
                                // better shutdown, although this might really be best.
                                // maybe a shutdown function which does universe.shut_down_next_tick = true;
                                // then later bellow actually shut down
                                // and with that function, I can also clean up stuff if I need to. if I even need to.
                            }
                            UniverseCommand::UnknownCommand => {

                            }
                        }
                    }

                    Some(event) = event_rx.recv() => {
                        // todo: universe events
                    }
                }

                (&mut universe).step();
            }
        });

        UniverseHandle {
            handle_id,
            commander_tx,
            universe_task_handle,
        }
    }
}

pub fn create_universe_handle() -> UniverseHandle {
    let universe = Universe::new();

    UniverseHandle::new(universe)
}