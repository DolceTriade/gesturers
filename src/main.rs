extern crate bincode;
extern crate circgr;
extern crate clap;
extern crate input;

use clap::{App, Arg, SubCommand};
use handler::Mode;
use std::thread;

mod collector;
mod handler;
mod libinput;

fn main() {
    let matches = App::new("GestureRS")
        .version("0.1.0")
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

    let mut handler = handler::Handler::new(mode, listener, std::path::Path::new("/tmp/gestures"));
    handler.run();
}
