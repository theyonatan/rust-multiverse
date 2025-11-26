use std::collections::HashSet;
use rand::{rng, Rng};
use tokio::sync::mpsc::UnboundedSender;
use crate::universe::{new_universe_id, UniverseId, UniverseIntent};

pub struct Universe {
    pub(crate) id: UniverseId,
    pub(crate) intent_tx: UnboundedSender<UniverseIntent>,
    pub(crate) executes: bool,
    pub(crate) tick: i32,
    pub(crate) hp: i32,
    pub(crate) enemies: HashSet<UniverseId>,
    pub(crate) brothers: HashSet<UniverseId>,
}

impl Universe {
    pub(crate) fn new(intent_tx: UnboundedSender<UniverseIntent>) -> Universe {
        let id = new_universe_id();

        Universe {
            id,
            intent_tx,
            executes: true,
            tick: 0,
            hp: 100,
            enemies: Default::default(),
            brothers: Default::default(),
        }
    }

    pub(crate) fn step(&mut self) {
        // death check
        if !self.executes { return; }

        // auto combat every 4 ticks
        if self.tick % 4 == 0 {
            self.attack_or_heal_random();
        }

        // tick
        self.tick += 1;
    }

    fn attack_or_heal_random(&mut self) {
        let mut rng = rng();
        let strength = rng.random_range(7..=20);

        if !self.enemies.is_empty() && rng.random_ratio(7, 10) {
            if let Some(&target) = self.enemies.iter().next() {
                let _ = self.intent_tx.send(UniverseIntent::Attack {target, damage: strength });
            }
        }

        if !self.brothers.is_empty() && rng.random_ratio(3, 10) {
            if let Some(&target) = self.brothers.iter().next() {
                let _ = self.intent_tx.send(UniverseIntent::Heal {target, amount: strength });
            }
        }
    }
    
    // helper utils for fighting stuff
    pub fn take_damage(&mut self, amount: i32) {
        self.hp -= amount;
        if self.hp <= 0 {
            self.hp = 0;
            self.executes = false;

            self.collapse();
        }
    }

    fn collapse(&mut self) {
        self.executes = false;
        let _ = self.intent_tx.send(UniverseIntent::Dead { target: self.id });
    }

    pub fn heal(&mut self, amount: i32) {
        self.hp += amount;
        if self.hp > 100 { self.hp = 100; }
    }

    pub fn is_enemy(&self, id: UniverseId) -> bool {
        self.enemies.contains(&id)
    }

    pub fn is_brother(&self, id: UniverseId) -> bool {
        self.brothers.contains(&id)
    }
}
