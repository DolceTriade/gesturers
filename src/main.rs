extern crate circgr;
extern crate input;

mod collector;
mod libinput;

fn main() {
    println!("Hello, world!");
    let mut collector = collector::Collector::new();
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
}
