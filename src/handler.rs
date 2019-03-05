use circgr::classifier::Classifier;
use circgr::gesture::Gesture;
use crossbeam::channel;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::process;

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Mode {
    Record(String),
    Recognize,
}

#[derive(Debug, Deserialize)]
struct Actions {
    default: Option<String>,
    app: Option<HashMap<String, String>>,
}

pub struct Handler {
    mode: Mode,
    gesture_listener: channel::Receiver<Gesture>,
    classifier: Classifier,
    template_path: PathBuf,
    actions: HashMap<String, Actions>,
}

impl Handler {
    pub fn new(mode: Mode, listener: channel::Receiver<Gesture>, config_path: &Path) -> Self {
        let mut classifier = Classifier::new();
        let mut gesture_path = PathBuf::from(config_path);
        let mut actions = HashMap::new();
        gesture_path.push("gestures");
        if mode == Mode::Recognize {
            println!("Reading config...");
            let mut config_path = PathBuf::from(config_path);
            config_path.push("Config.toml");
            if config_path.exists() && config_path.is_file() {
                let config_str = std::fs::read(&config_path)
                    .expect(&format!("Error reading {:?}", &config_path));
                actions = toml::from_slice(&config_str).unwrap();
            }
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
            actions: actions,
        }
    }

    pub fn run(&mut self) {
        const PER_TRACE_THRESHOLD: usize = 100;
        let empty = HashMap::new();
        let display = wlib::Display::open().unwrap();
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
                _ => match &self.classifier.classify(&gesture) {
                    Some((name, score)) => {
                        let window = get_window_at_cursor(&display);
                        println!(
                            "Got gesture {} with score {} in {:?}",
                            &name, &score, &window
                        );
                        if *score > PER_TRACE_THRESHOLD * gesture.traces.len() {
                            println!("Score too low...");
                            continue;
                        }
                        println!("Gesture happeened in {:?}", &window);
                        let mut action = "".to_string();
                        match self.actions.get(name) {
                            Some(actions) => {
                                match actions
                                    .app
                                    .as_ref()
                                    .unwrap_or(&empty)
                                    .get(&window.unwrap_or("".to_string()))
                                {
                                    Some(a) => action = a.to_string(),
                                    None => match &actions.default {
                                        Some(a) => action = a.to_string(),
                                        None => {}
                                    },
                                }
                            }
                            _ => {}
                        }
                        if !action.is_empty() {
                            println!("Running: {}", &action);
                            match process::Command::new("/bin/sh")
                                .args(&["-c", &action])
                                .spawn()
                            {
                                Ok(_) => {}
                                Err(err) => {
                                    println!("Error executing \"{}\": {:?}", &action, &err);
                                }
                            }
                        }
                    }
                    _ => {}
                },
            }
        }
    }
}

fn get_window_at_cursor(display: &wlib::Display) -> Option<String> {
    let w = display.focus().unwrap().unwrap();
    w.class_name()
}
