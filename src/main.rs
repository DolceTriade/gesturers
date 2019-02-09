extern crate input;

mod libinput;

fn main() {
    println!("Hello, world!");
    let mut ctx = libinput::init();
    loop {
        ctx.ready.recv().unwrap();
        ctx.libinput.dispatch().unwrap();
        for event in &mut ctx.libinput {
            match event {
                input::Event::Touch(touch_event) => {
                    println!("Touch event {:?}", touch_event);
                }
                _ => {}
            }
        }
    }
}
