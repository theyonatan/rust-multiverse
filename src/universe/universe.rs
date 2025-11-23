use std::collections::{HashMap, HashSet};
use rand::Rng;
use crossterm::style::Color;
use rand::seq::IteratorRandom;
use tokio::sync::mpsc::UnboundedSender;
use crate::universe::{new_universe_id, UniverseId, UniverseEvent};
use crate::universe::universe_event::UniverseOutboundEvent;


pub struct Universe {
    pub(crate) id: UniverseId,
    pub(crate) name: String,
    pub(crate) color: Color,
    executes: bool,
    tick: u64,
    pub(crate) hp: i32,
    enemies: HashSet<UniverseId>,
    brothers: HashSet<UniverseId>,
    outbound_tx: Option<UnboundedSender<UniverseOutboundEvent>>,
    known_names: HashMap<UniverseId, String>,
}

impl Universe {
    // ... new, random_color, set_outbound_tx remain same ...
    pub(crate) fn new(name: String) -> Universe {
        let id = new_universe_id();
        let color = Self::random_color();

        Universe {
            id,
            name,
            color,
            tick: 0,
            hp: 100,
            executes: true,
            enemies: Default::default(),
            brothers: Default::default(),
            outbound_tx: None,
            known_names: HashMap::new(),
        }
    }

    pub fn set_outbound_tx(&mut self, tx: UnboundedSender<UniverseOutboundEvent>) {
        self.outbound_tx = Some(tx);
    }

    fn random_color() -> Color {
        let colors = [Color::Red, Color::Green, Color::Blue, Color::Yellow, Color::Magenta, Color::Cyan];
        let mut rng = rand::thread_rng();
        colors[rng.gen_range(0..colors.len())]
    }

    pub(crate) fn step(&mut self) {
        if !self.executes || self.hp <= 0 { return; }
        self.tick += 1;
        let mut rng = rand::thread_rng();

        // 1. Random chance to interact
        if rng.gen_bool(0.25) {
            self.decide_action();
        }
    }

    fn decide_action(&mut self) {
        if self.outbound_tx.is_none() { return; }
        let tx = self.outbound_tx.as_ref().unwrap();
        let mut rng = rand::thread_rng();

        // 30% chance to heal a brother, 70% chance to attack an enemy → death becomes inevitable
        if rng.gen_bool(0.70) {
            if let Some(&enemy_id) = self.enemies.iter().choose(&mut rng) {
                let damage = rng.gen_range(12..28); // Bigger hits
                let target_name = self.known_names.get(&enemy_id)
                    .map(|s| s.as_str())
                    .unwrap_or("unknown");

                self.log_internal(format!("attacking → [{}] -{} dmg", target_name, damage));

                let _ = tx.send(UniverseOutboundEvent::MessagePeer {
                    target_id: enemy_id,
                    event: UniverseEvent::Shatter(damage), // We'll modify Shatter to carry damage
                });
            }
        } else {
            if let Some(&friend_id) = self.brothers.iter().choose(&mut rng) {
                let friend_name = self.known_names
                    .get(&friend_id)
                    .map(|s| s.as_str())
                    .unwrap_or("brother");

                self.log_internal(format!("healing → [{friend_name}]"));

                let _ = tx.send(UniverseOutboundEvent::MessagePeer {
                    target_id: friend_id,
                    event: UniverseEvent::Heal(8),
                });
            }
        }
    }

    pub fn meet_peer(&mut self, id: UniverseId, name: &str) {
        if id == self.id { return; }
        let mut rng = rand::thread_rng();
        let is_friend = rng.gen_bool(0.5);

        self.known_names.insert(id, name.to_string()); // ← Remember the name!

        if is_friend {
            self.brothers.insert(id);
            self.log_internal(format!("alliance formed → [{}]", name));
        } else {
            self.enemies.insert(id);
            self.log_internal(format!("hostility declared → [{}]", name));
        }
    }

    pub fn handle_shatter(&mut self, damage: u32) {
        self.hp -= damage as i32;
        self.log_internal(format!("SHATTERED! -{} HP → {} left", damage, self.hp.max(0)));

        if self.hp <= 0 {
            self.die();
        }
    }

    pub fn handle_heal(&mut self, amount: u32) {
        if self.hp > 0 {
            let old_hp = self.hp;
            self.hp += amount as i32;
            if self.hp > 100 { self.hp = 100; }
            self.log_internal(format!("healed +{} HP ({} → {})", amount, old_hp, self.hp));
        }
    }

    pub(crate) fn die(&mut self) {
        self.executes = false;
        self.log_internal("COLLAPSED. Universe destroyed.".to_string());
        if let Some(tx) = &self.outbound_tx {
            let _ = tx.send(UniverseOutboundEvent::BroadcastDeath(self.id));
            // Tell everyone I died
            let _ = tx.send(UniverseOutboundEvent::BroadcastPeerDied(self.id));
        }
    }

    pub(crate) fn log_internal(&self, msg: String) {
        if let Some(tx) = &self.outbound_tx {
            let _ = tx.send(UniverseOutboundEvent::Log {
                name: self.name.clone(),
                color: self.color, // Copy Color
                message: msg
            });
        }
    }

    pub fn report_state(&self) {
        if let Some(tx) = &self.outbound_tx {
            let _ = tx.send(UniverseOutboundEvent::Log {
                name: self.name.clone(),
                color: self.color,
                message: format!("STATE → HP: {} | Tick: {} | Alive: {}",
                                 self.hp.max(0), self.tick, self.hp > 0)
            });
        }
    }
}