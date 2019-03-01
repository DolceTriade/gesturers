use circgr::gesture::{Gesture, Point, PointBuilder};
use crossbeam::channel;
use input::event::touch::{TouchEvent, TouchEventPosition, TouchEventSlot, TouchEventTrait};
use std::collections::{HashMap, HashSet};

const RESAMPLE_SIZE: u32 = 64;

pub struct Collector {
    pub gesture_listener: channel::Receiver<Gesture>,
    sender: channel::Sender<Gesture>,
    raw_input: HashMap<u32, Vec<Point>>,
    fingers: HashSet<u32>,
}

impl Collector {
    pub fn new() -> Collector {
        let (s, r) = channel::unbounded();
        Collector {
            gesture_listener: r,
            sender: s,
            raw_input: HashMap::new(),
            fingers: HashSet::new(),
        }
    }
    pub fn handle_event(&mut self, event: &TouchEvent) {
        match event {
            TouchEvent::Up(up) => {
                self.fingers.remove(&up.slot().unwrap_or(1));
            }
            TouchEvent::Down(down) => {
                let id = down.slot().unwrap_or(1);
                self.fingers.insert(id.clone());
                self.raw_input
                    .entry(id)
                    .or_insert(Vec::new())
                    .push(event_position(down));
            }
            TouchEvent::Motion(motion) => {
                let id = motion.slot().unwrap_or(1);
                self.fingers.insert(id.clone());
                self.raw_input
                    .entry(id)
                    .or_insert(Vec::new())
                    .push(event_position(motion));
            }
            _ => {}
        }

        if self.fingers.is_empty() && !self.raw_input.is_empty() {
            let gesture = Gesture::new(&self.raw_input, RESAMPLE_SIZE);
            self.raw_input.clear();
            self.sender.send(gesture).unwrap();
        }
    }
}

fn event_position<T: TouchEventPosition + TouchEventTrait>(event: &T) -> Point {
    PointBuilder::default()
        .x(event.x())
        .y(event.y())
        .timestamp(event.time_usec())
        .build()
        .unwrap()
}
