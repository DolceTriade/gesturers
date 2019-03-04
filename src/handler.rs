use circgr::classifier::Classifier;
use circgr::gesture::Gesture;
use crossbeam::channel;
use std::path::{Path, PathBuf};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Mode {
    Record(String),
    Recognize,
}

pub struct Handler {
    mode: Mode,
    gesture_listener: channel::Receiver<Gesture>,
    classifier: Classifier,
    template_path: PathBuf,
}

impl Handler {
    pub fn new(mode: Mode, listener: channel::Receiver<Gesture>, config_path: &Path) -> Self {
        let mut classifier = Classifier::new();
        let mut gesture_path = PathBuf::from(config_path);
        gesture_path.push("gestures");
        if mode == Mode::Recognize {
            println!("Adding templates...");
            let dir = std::fs::read_dir(&gesture_path)
                .expect(&format!("Error reading {:?}", &gesture_path));
            for entry_or in dir {
                let entry = entry_or.expect("Error reading entry");
                println!("Checking {:?}", entry);
                let bytes = std::fs::read(entry.path())
                    .expect(&format!("Error reading {:?}", entry.path()));
                let gesture_or: bincode::Result<Gesture> = bincode::deserialize(&bytes);
                match gesture_or {
                    Ok(gesture) => classifier.add_template(gesture),
                    Err(err) => {
                        eprintln!("Error decoding gesture in {:?}: {:?}", entry.path(), err)
                    }
                }
            }
        }
        Handler {
            mode: mode,
            gesture_listener: listener,
            classifier: classifier,
            template_path: gesture_path,
        }
    }

    pub fn run(&mut self) {
        loop {
            let mut gesture = self.gesture_listener.recv().unwrap();
            match &self.mode {
                Mode::Record(name) => {
                    gesture.name = name.to_string();
                    let mut path = std::path::PathBuf::new();
                    path.push(&self.template_path);
                    path.push(name);
                    std::fs::write(path.as_path(), bincode::serialize(&gesture).unwrap()).unwrap();
                    break;
                }
                _ => {
                    println!("Doing stuff: {:?}", &self.classifier.classify(&gesture));
                }
            }
        }
    }
}
