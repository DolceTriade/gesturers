use circgr::gesture::{Gesture, Point, PointBuilder};
use crossbeam::channel;
use input::event::touch::{TouchEvent, TouchEventPosition, TouchEventSlot, TouchEventTrait};
use std::collections::{HashMap, HashSet};

const RESAMPLE_SIZE: u32 = 64;

pub struct Collector<'s> {
    sender: channel::Sender<Gesture>,
    raw_input: HashMap<u32, Vec<Point>>,
    fingers: HashSet<u32>,
    screen: &'s wlib::Screen<'s>,
}

impl<'s> Collector<'s> {
    pub fn new(screen: &'s wlib::Screen, sender: channel::Sender<Gesture>) -> Self {
        Collector {
            sender: sender,
            raw_input: HashMap::new(),
            fingers: HashSet::new(),
            screen: screen,
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
                    .push(event_position(self.screen, down));
            }
            TouchEvent::Motion(motion) => {
                let id = motion.slot().unwrap_or(1);
                self.fingers.insert(id.clone());
                self.raw_input
                    .entry(id)
                    .or_insert(Vec::new())
                    .push(event_position(self.screen, motion));
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

fn event_position<'s, T: TouchEventPosition + TouchEventTrait>(
    screen: &'s wlib::Screen,
    event: &T,
) -> Point {
    PointBuilder::default()
        .x(event.x_transformed(screen.width()))
        .y(event.y_transformed(screen.height()))
        .timestamp(event.time_usec())
        .build()
        .unwrap()
}
