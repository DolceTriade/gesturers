#[macro_use]
extern crate clap;
extern crate input;
extern crate libinput;

use clap::{App, Arg, SubCommand};
use input::event::DeviceEvent;
use input::event::EventTrait;
use input::Event;

fn main() -> Result<(), String> {
    let matches = App::new("calibration_matrix")
        .version("0.1.0")
        .arg(
            Arg::with_name("device")
                .short("d")
                .value_name("DEVICE")
                .help("libinput device name.")
                .takes_value(true)
                .required(true)
        )
        .subcommand(
            SubCommand::with_name("get")
                .about("Get the calibration matrix for a device.")
        )
        .subcommand(
            SubCommand::with_name("reset")
                .about("Reset the calibration matrix for a device.")
        )
        .subcommand(
            SubCommand::with_name("set")
                .about("Set the calibration matrix for device.")
                .arg(
                    Arg::with_name("values")
                        .short("v")
                        .value_name("VALUES")
                        .help("6 floats for the calibration matrix. See libinput documentation for info.")
                        .takes_value(true)
                        .multiple(true)
                        .required(true)
                )
        )
        .get_matches();
    let device_name = matches.value_of("device").unwrap();
    let mut ctx = libinput::init().unwrap();
    let mut found = false;
    ctx.libinput.dispatch().unwrap();
    for event in &mut ctx.libinput {
        match event {
            Event::Device(device_event) => match device_event {
                DeviceEvent::Added(device_added_event) => {
                    let mut device = device_added_event.device();
                    if device.name() == device_name {
                        found = true;
                        if !device.config_calibration_has_matrix() {
                            return Err(format!(
                                "{} cannot be configured with a calibration matrix.",
                                device.name()
                            ));
                        }
                        match matches.subcommand() {
                            ("get", Some(_)) => {
                                println!("{:?}", device.config_calibration_matrix());
                            }
                            ("reset", Some(_)) => {
                                let matrix = device.config_calibration_default_matrix().unwrap();
                                device.config_calibration_set_matrix(matrix).unwrap();
                            }
                            ("set", Some(m)) => {
                                if m.occurrences_of("values") != 6 {
                                    return Err("Must have 6 values!".to_string());
                                }
                                let matrix = values_t!(m.values_of("values"), f32)
                                    .unwrap_or_else(|e| e.exit());
                                let mut matrix_arr: [f32; 6] = [0.0_f32; 6];
                                matrix_arr.copy_from_slice(&matrix);
                                device.config_calibration_set_matrix(matrix_arr).unwrap();
                            }
                            _ => panic!(),
                        }
                        break;
                    }
                }
                _ => {
                    break;
                }
            },
            _ => {
                break;
            }
        }
    }
    if found {
        return Ok(());
    } else {
        return Err(format!("Could not find device: {}", device_name));
    }
}
