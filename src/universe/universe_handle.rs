use std::time::Duration;
use tokio::sync::mpsc::{Sender, unbounded_channel, channel, UnboundedReceiver};
use tokio::time::{interval, MissedTickBehavior};
use crate::universe::intent::UniverseIntent;
use crate::universe::id::UniverseId;
use crate::universe::relationship::Relationship;
use crate::universe::universe::Universe;
use crate::universe::universe_command::UniverseCommand;
use crate::universe::universe_event::UniverseEvent;

pub struct UniverseHandle {
    pub(crate) handle_id: UniverseId,
    pub(crate) commander_tx: Sender<UniverseCommand>,
    pub(crate) universe_task_handle: tokio::task::JoinHandle<()>,
    pub(crate) intent_rx: UnboundedReceiver<UniverseIntent>,
}

impl UniverseHandle {
    fn new(mut universe: Universe, intent_rx: UnboundedReceiver<UniverseIntent>) -> UniverseHandle {
        let handle_id = universe.id.clone();

        let (commander_tx, mut command_rx) = channel::<UniverseCommand>(10);

        let universe_task_handle = tokio::spawn(async move{
            let mut ticker = interval(Duration::from_millis(40)); // 0.4s per step
            ticker.set_missed_tick_behavior(MissedTickBehavior::Skip);

            // todo: move to logger: println!("Spawned universe async instance!");

            loop {
                tokio::select! {
                    // commands from supervisor
                    Some(command) = command_rx.recv() => {
                        // todo: move to logger: println!("Given universe command: {:?}", command);
                        handle_given_command(&command, &mut universe);
                    }

                    // other -> nothing is pending, perform step
                    _ = ticker.tick() => {
                        // step, while waiting for rx
                    }

                }

                if universe.executes == true && universe.hp > 0 {
                    (&mut universe).step();
                } else if universe.hp <= 0 {
                    // todo: alert supervisor that universe died, which alerts other universes as well.
                    break;
                }
            }
        });

        UniverseHandle {
            handle_id,
            commander_tx,
            universe_task_handle,
            intent_rx,
        }
    }
}

pub fn create_universe_handle() -> UniverseHandle {
    let (intent_tx, intent_rx) = unbounded_channel::<UniverseIntent>();

    let universe = Universe::new(intent_tx);

    UniverseHandle::new(universe, intent_rx)
}

fn handle_given_command(command: &UniverseCommand, universe: &mut Universe) {
    match command {
        UniverseCommand::Start => {
            universe.executes = true;
        }
        UniverseCommand::Stop => {
            universe.executes = false;
        }
        UniverseCommand::InjectEvent(event) => {
            handle_given_event(event, universe);
        }
        UniverseCommand::RequestState() => {
            // todo: logger thread
            // should push a message to a queue that goes to the output stream
            // or log, which a logger then logs on a different thread.
            // good usage for threads here, use a logger, this pushes to logger,
            // on a different thread a logger exists and prints all from queue.
        }
        UniverseCommand::SetRelationship(id, relationship) => {
            match relationship {
                Relationship::Enemy => { universe.enemies.insert(id.clone()); }
                Relationship::Brother => { universe.brothers.insert(id.clone()); }
            } // todo: make sure supervisor decides if they are brothers or enemies and sends this
        }
        UniverseCommand::Shutdown => {

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

fn handle_given_event(event: &UniverseEvent, universe: &mut Universe) {
    match event {
        UniverseEvent::Shatter(strength) => {
            universe.take_damage(*strength);
        }
        UniverseEvent::Crash => {
            universe.take_damage(999);  // insta-kill
        }
        UniverseEvent::Heal(strength) => {
            universe.heal(*strength);
        }
        _ => {}
    }
}