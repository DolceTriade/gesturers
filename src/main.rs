extern crate bincode;
extern crate circgr;
extern crate clap;
extern crate dirs;
extern crate libinput;
extern crate toml;

use clap::{App, Arg, SubCommand};
use handler::Mode;
use std::thread;

mod collector;
mod handler;

fn main() {
    let matches = App::new("GestureRS")
        .version("0.1.0")
        .arg(
            Arg::with_name("conf_path")
                .short("c")
                .value_name("CONFIG_PATH")
                .help("Config path for actions and gestures."),
        )
        .subcommand(
            SubCommand::with_name("record")
                .about("Record a new gesture")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .value_name("NAME")
                        .help("Name for the recorded gesture")
                        .required(true),
                ),
        )
        .get_matches();
    let mut collector = collector::Collector::new();
    let listener = collector.gesture_listener.clone();
    let mode = match matches.subcommand_matches("record") {
        Some(m) => Mode::Record(m.value_of("name").unwrap().to_string()),
        _ => Mode::Recognize,
    };

    thread::spawn(move || {
        let mut ctx = libinput::init().unwrap();
        loop {
            ctx.ready.recv().unwrap();
            ctx.libinput.dispatch().unwrap();
            for event in &mut ctx.libinput {
                match event {
                    input::Event::Touch(touch_event) => {
                        collector.handle_event(&touch_event);
                    }
                    _ => {}
                }
            }
        }
    });

    let config_path = match matches.value_of("conf_path") {
        Some(path) => std::path::PathBuf::from(path),
        None => {
            let mut path = dirs::config_dir().unwrap();
            path.push("gesturers");
            path
        }
    };

    if !config_path.exists() {
        std::fs::create_dir_all(&config_path)
            .expect(&format!("Unable to create path: {:?}", &config_path));
    }
    if !config_path.is_dir() {
        eprintln!("config path must be a directory!");
        panic!();
    }
    let mut gesture_dir = config_path.clone();
    gesture_dir.push("gestures");
    std::fs::create_dir_all(&gesture_dir).expect(&format!(
        "Unable to create gesture path: {:?}",
        &gesture_dir
    ));

    let mut handler = handler::Handler::new(mode, listener, &config_path);
    handler.run();
}
