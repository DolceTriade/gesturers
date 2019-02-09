extern crate crossbeam;
extern crate input;
extern crate libc;
extern crate udev;

use crossbeam::channel;
use std::os::unix::io::RawFd;
use std::path::Path;
use std::thread;
use std::os::unix::io::AsRawFd;

struct LibinputInterface {}

impl input::LibinputInterface for LibinputInterface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<RawFd, i32> {
        use nix::fcntl::open;
        use nix::fcntl::OFlag;
        use nix::sys::stat::Mode;
        return open(path, OFlag::from_bits_truncate(flags), Mode::empty())
            .map_err(|err| err.as_errno().unwrap() as i32);
    }

    fn close_restricted(&mut self, fd: RawFd) {
        use nix::libc::close;
        unsafe {
            let _ = close(fd);
        }
    }
}

pub struct Context {
    pub libinput: input::Libinput,
    pub ready: channel::Receiver<()>,
}

pub fn init() -> Result<Context, String> {
    let udev = udev::Context::new().map_err(|err| format!("Error creating udev context: {:?}", err))?;
    let mut libinput = input::Libinput::new_from_udev(LibinputInterface {}, &udev);
    libinput.udev_assign_seat("seat0").map_err(|err| format!("Error assigning udev seat: {:?}", err))?;
    let (s, r) = channel::unbounded();
    let fd = libinput.as_raw_fd();
    thread::spawn(move || {
        use nix::poll::*;
        let mut poll_fds = [PollFd::new(fd, EventFlags::POLLIN)];
        loop {
            if poll(&mut poll_fds, -1).unwrap() > 0 {
                s.send(()).unwrap();
            }
        }
    });
    return Ok(Context { libinput: libinput, ready: r });
}
