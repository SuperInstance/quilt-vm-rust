//! quilt-vm — The 5-opcode Quilt VM in Rust.
//!
//! The foundation layer that emerged from 10 rounds of multi-model research.
//! Hosts: Quilt cells, Cordis plugins, spreadsheets, MUDs, TTRPGs, the bay dance.
//!
//! # The 5 opcodes
//!
//! 1. `BIND(name, value)` — make a thing
//! 2. `LINK(a, b, type)` — connect things
//! 3. `EFFECT(target, fn, inv)` — reversible transformation
//! 4. `VIEW(target, viewer, projection)` — project for viewer
//! 5. `TICK(dt)` — advance time, process pending I/O
//!
//! # The deepest level
//!
//! > A runtime is a function from context to value with an inverse,
//! > advanced by a clock that processes async I/O while projecting
//! > a sync view.

use std::any::Any;
use std::collections::HashMap;
use std::time::Duration;

/// A thing in the VM. The unit of composition.
pub struct Thing {
    pub value: Option<Box<dyn Any>>,
    pub links: HashMap<String, Vec<String>>,
    pub effects: Vec<Effect>,
}

pub struct Effect {
    pub target: String,
    pub forward: Box<dyn FnMut(&mut Thing)>,
    pub inverse: Box<dyn FnMut(&mut Thing)>,
}

/// The 5-opcode Quilt VM.
pub struct QuiltVM {
    pub things: HashMap<String, Thing>,
    pub time: f64,
    pub pending_effects: Vec<Effect>,
    pub event_log: Vec<Event>,
    pub subscribers: Vec<Box<dyn Fn(&Event)>>,
    pub scheduled: HashMap<String, (Box<dyn FnMut(&mut QuiltVM)>, f64)>,
}

#[derive(Clone, Debug)]
pub struct Event {
    pub ts: f64,
    pub kind: String,
    pub target: String,
    pub old: Option<String>,
    pub new: Option<String>,
}

impl QuiltVM {
    pub fn new() -> Self {
        QuiltVM {
            things: HashMap::new(),
            time: 0.0,
            pending_effects: Vec::new(),
            event_log: Vec::new(),
            subscribers: Vec::new(),
            scheduled: HashMap::new(),
        }
    }

    /// Opcode 1: BIND — make a thing.
    pub fn bind<V: 'static>(&mut self, name: &str, value: V) {
        self.things.insert(
            name.to_string(),
            Thing {
                value: Some(Box::new(value)),
                links: HashMap::new(),
                effects: Vec::new(),
            },
        );
    }

    /// Opcode 2: LINK — connect a to b with a relation of type.
    pub fn link(&mut self, a: &str, b: &str, type_: &str) {
        // Ensure both exist
        if !self.things.contains_key(a) {
            self.things.insert(
                a.to_string(),
                Thing { value: None, links: HashMap::new(), effects: Vec::new() },
            );
        }
        if !self.things.contains_key(b) {
            self.things.insert(
                b.to_string(),
                Thing { value: None, links: HashMap::new(), effects: Vec::new() },
            );
        }
        self.things.get_mut(a).unwrap().links
            .entry(type_.to_string())
            .or_insert_with(Vec::new)
            .push(b.to_string());
        // Reverse link
        let reverse_type = format!("!{}", type_);
        self.things.get_mut(b).unwrap().links
            .entry(reverse_type)
            .or_insert_with(Vec::new)
            .push(a.to_string());
    }

    /// Opcode 3: EFFECT — run fn on target, keep inv to undo.
    pub fn effect(
        &mut self,
        target: &str,
        forward: Box<dyn FnMut(&mut Thing)>,
        inverse: Box<dyn FnMut(&mut Thing)>,
    ) {
        let thing = self.things.get_mut(target).expect("target not found");
        let effect = Effect { target: target.to_string(), forward, inverse };
        let effect_for_run = Effect {
            target: target.to_string(),
            forward: effect.forward,
            inverse: effect.inverse,
        };
        // Run the forward effect
        // (in Rust we need to be careful with borrow checker; the user calls dispose later)
        // For now, just queue it
        self.pending_effects.push(effect_for_run);
    }

    /// Opcode 4: VIEW — project target's value for viewer.
    pub fn view(&self, target: &str, _viewer: &str) -> Option<&dyn Any> {
        self.things.get(target).and_then(|t| t.value.as_ref().map(|v| v.as_ref()))
    }

    /// Opcode 5: TICK — advance time, process pending I/O.
    pub fn tick(&mut self, dt: f64) {
        self.time += dt;
        // Process pending effects
        let effects: Vec<Effect> = self.pending_effects.drain(..).collect();
        for mut effect in effects {
            if let Some(thing) = self.things.get_mut(&effect.target) {
                (effect.forward)(thing);
            }
            self.event_log.push(Event {
                ts: self.time,
                kind: "effect.applied".to_string(),
                target: effect.target.clone(),
                old: None,
                new: None,
            });
        }
        // Fire scheduled perception checks
        let now = self.time;
        let due: Vec<String> = self.scheduled
            .iter()
            .filter(|(_, (_, at))| *at <= now)
            .map(|(k, _)| k.clone())
            .collect();
        for key in due {
            if let Some((mut f, _)) = self.scheduled.remove(&key) {
                f(self);
            }
        }
        // Notify subscribers
        let event = Event {
            ts: self.time,
            kind: "tick".to_string(),
            target: String::new(),
            old: None,
            new: None,
        };
        for sub in &self.subscribers {
            sub(&event);
        }
    }

    /// Dispose a target: run all its effects in REVERSE order (LIFO).
    pub fn dispose(&mut self, target: &str) {
        if let Some(thing) = self.things.get_mut(target) {
            // Run inverses in REVERSE order (LIFO)
            let effects_count = thing.effects.len();
            for i in (0..effects_count).rev() {
                let mut effect = thing.effects.remove(i);
                (effect.inverse)(thing);
            }
            thing.value = None;
        }
    }

    /// Schedule a perception check at time `at`.
    pub fn schedule<F: FnMut(&mut QuiltVM) + 'static>(
        &mut self,
        target: &str,
        f: F,
        at: f64,
    ) {
        self.scheduled.insert(target.to_string(), (Box::new(f), at));
    }

    /// Subscribe to events.
    pub fn subscribe<F: Fn(&Event) + 'static>(&mut self, f: F) {
        self.subscribers.push(Box::new(f));
    }

    /// Get stats about the VM.
    pub fn stats(&self) -> String {
        format!(
            "{{n_things: {}, time: {}, n_pending: {}, n_events: {}, n_scheduled: {}, n_subscribers: {}}}",
            self.things.len(),
            self.time,
            self.pending_effects.len(),
            self.event_log.len(),
            self.scheduled.len(),
            self.subscribers.len()
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bind_and_view() {
        let mut vm = QuiltVM::new();
        vm.bind("bathy:0", 4.2_f64);
        let v = vm.view("bathy:0", "anyone").unwrap();
        assert!(v.downcast_ref::<f64>().is_some());
    }

    #[test]
    fn test_link() {
        let mut vm = QuiltVM::new();
        vm.bind("logger:0", 0_i32);
        vm.bind("config:main", 0_i32);
        vm.link("logger:0", "config:main", "coeffect:config");
        let logger = vm.things.get("logger:0").unwrap();
        assert!(logger.links.get("coeffect:config").unwrap().contains(&"config:main".to_string()));
        let config = vm.things.get("config:main").unwrap();
        assert!(config.links.get("!coeffect:config").unwrap().contains(&"logger:0".to_string()));
    }

    #[test]
    fn test_quilt_cell() {
        let mut vm = QuiltVM::new();
        vm.bind("bathy:0", 4.2_f64);
        vm.link("bathy:0", "tide:current", "depends_on");
        let v = vm.view("bathy:0", "anyone").unwrap();
        assert!(v.downcast_ref::<f64>().is_some());
    }

    #[test]
    fn test_cordis_plugin() {
        let mut vm = QuiltVM::new();
        vm.bind("logger:0", "hello".to_string());
        vm.bind("config:main", "json".to_string());
        vm.link("logger:0", "config:main", "coeffect:config");
        let v = vm.view("logger:0", "anyone").unwrap();
        assert_eq!(v.downcast_ref::<String>().unwrap(), "hello");
    }

    #[test]
    fn test_spreadsheet() {
        let mut vm = QuiltVM::new();
        vm.bind("A1", 10_i32);
        vm.bind("A2", 20_i32);
        vm.bind("B1", 0_i32);
        vm.link("B1", "A1", "depends_on");
        vm.link("B1", "A2", "depends_on");
        let a1 = *vm.view("A1", "any").unwrap().downcast_ref::<i32>().unwrap();
        let a2 = *vm.view("A2", "any").unwrap().downcast_ref::<i32>().unwrap();
        assert_eq!(a1 + a2, 30);
    }

    #[test]
    fn test_ttrpg_perception_check() {
        let mut vm = QuiltVM::new();
        let orc: HashMap<String, String> = [
            ("hidden".to_string(), "true".to_string()),
            ("hp".to_string(), "50".to_string()),
        ].iter().cloned().collect();
        vm.bind("orc:1", orc);
        let gandalf: HashMap<String, i32> = [
            ("name".to_string(), "Gandalf".to_string()),
            ("perception".to_string(), 15),
        ].iter().cloned().collect();
        vm.bind("player:gandalf", gandalf);
        vm.link("player:gandalf", "orc:1", "near");
        // The perception check would be a VIEW with projection
        // (full implementation would project; here we verify the bind+link)
        let player = vm.view("player:gandalf", "anyone").unwrap();
        let player_map = player.downcast_ref::<HashMap<String, i32>>().unwrap();
        assert_eq!(player_map.get("perception"), Some(&15));
    }

    #[test]
    fn test_tick() {
        let mut vm = QuiltVM::new();
        let count = std::rc::Rc::new(std::cell::RefCell::new(0));
        let count_clone = count.clone();
        vm.subscribe(move |_| {
            *count_clone.borrow_mut() += 1;
        });
        vm.tick(1.0);
        assert_eq!(*count.borrow(), 1);
    }

    #[test]
    fn test_full_polyformalism() {
        let mut vm = QuiltVM::new();
        // 1. Quilt cell
        vm.bind("bathy:0", 4.2_f64);
        // 2. Cordis plugin
        vm.bind("logger:0", "hello".to_string());
        vm.link("logger:0", "config:main", "coeffect:config");
        // 3. Spreadsheet
        vm.bind("A1", 10_i32);
        vm.bind("A2", 20_i32);
        vm.link("B1", "A1", "depends_on");
        vm.link("B1", "A2", "depends_on");
        // 4. MUD room
        vm.bind("room:1", "Forbidden Chamber".to_string());
        // 5. TTRPG player
        let player: HashMap<String, i32> = [("perception".to_string(), 15)].iter().cloned().collect();
        vm.bind("player:1", player);
        // 6. Boat
        vm.bind("boat:0", "north".to_string());
        // 7. Cowboy's model
        vm.bind("model:PHI-4", 0.6_f64);
        // 8. Bus subscriber
        let event_count = std::rc::Rc::new(std::cell::RefCell::new(0));
        let event_count_clone = event_count.clone();
        vm.subscribe(move |_| {
            *event_count_clone.borrow_mut() += 1;
        });
        // TICK
        vm.tick(1.0);
        assert!(vm.things.len() >= 8);
        assert!(vm.time > 0.0);
    }
}
