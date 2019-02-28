extern crate circgr;
extern crate clap;
extern crate input;

use clap::{App, Arg, SubCommand};
use std::thread;

mod collector;
mod libinput;

enum Mode {
    Record,
    Normal,
}

fn main() {
    let matches = App::new("GestureRS")
        .version("0.1.0")
        .subcommand(
            SubCommand::with_name("record")
                .about("Record a new gesture")
                .arg(
                    Arg::with_name("name")
                        .short("n")
                        .help("Name for the recorded gesture")
                        .required(true),
                ),
        )
        .get_matches();
    let mut classifier = circgr::classifier::Classifier::new();
    let mut collector = collector::Collector::new();
    let mut ctx = libinput::init().unwrap();
    let listener = collector.gesture_listener.clone();
    let mode = match matches.subcommand_matches("record") {
        Some(_) => Mode::Record,
        _ => Mode::Normal,
    };
    if mode ==
    thread::spawn(move || {
        let gesture = listener.recv().unwrap();
        if mode == Mode::Record
        println!("Got gesture: {:?}", &gesture);
    });

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
}
