use gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use log::error;
use nix::poll::{poll, PollFd, PollFlags};
use smallvec::SmallVec;
use std::os::unix::io::AsRawFd;

use crate::error::Error;
use sensor_value::{SensorValue, Button, Press};
use crossbeam_channel::Sender;
use std::thread;

fn button_from_offset(offset: usize) -> Option<Button> {
    use Button::*;
    match offset {
        16 => Some(LampLeft),
        12 => Some(LampMid),
        13 => Some(LampRight),

        //buttons on desk
        27 => Some(DeskTop),
        22 => Some(DeskMid),
        18 => Some(DeskBottom),

        23 => Some(DeskLeftMost),
        24 => Some(DeskLeft),
        26 => Some(DeskRight),
        17 => Some(DeskRightMost),

        _ => None,
    }
}

fn press_from(offset: usize, duration: u64) -> Option<Press> {
    Some(Press {
        button: button_from_offset(offset)?,
        duration: duration as u16,
    })
}

// when pressing 22, 24 is often activated
// 23 works perfectly
// 24 also works perfectly
const MILLIS: u64 = 1_000_000; //nano to milli
const N_LINES: usize = 54;
fn detect_and_handle(chip: &mut Chip, s: Sender<SensorValue>) -> Result<(), Error> {
    let offsets: [u32; 10] = [27, 22, 18, 23, 24, 26, 17, 16, 12, 13];
    let (mut evt_handles, mut pollables) = configure_watching(chip, &offsets)?;
    thread::spawn(move || {
        let mut last_high = [0u64; N_LINES];
        let mut last_state = [0u8; N_LINES];

        loop {
            if poll(&mut pollables, -1).unwrap() != 0 {
                let key_presses = process_event(
                    &pollables,
                    &mut evt_handles,
                    &mut last_high,
                    &mut last_state,
                );
                for (offset, down_duration) in key_presses {
                    if down_duration > 10 * MILLIS {
                        //debounce
                        if let Some(p) = press_from(offset, down_duration) {
                            s.send(SensorValue::ButtonPress(p)).unwrap();
                        }
                    }
                }
            }
        }
    });
    Ok(())
}

///returns keys that where held as the time they where held in nanoseconds
fn process_event(
    pollables: &Vec<PollFd>,
    evt_handles: &mut Vec<LineEventHandle>,
    last_rising: &mut [u64],
    last_state: &mut [u8],
) -> SmallVec<[(usize, u64); 64]> {
    let mut key_presses = SmallVec::<[(usize, u64); 64]>::new();
    for i in 0..pollables.len() {
        if let Some(poll_res) = pollables[i].revents() {
            let h = &mut evt_handles[i];
            if poll_res.contains(PollFlags::POLLIN) {
                let value = h.get_value().unwrap();
                let event = h.get_event().unwrap();
                let offset = h.line().offset() as usize;

                if value == 1 && last_state[offset] == 0 {
                    //rising
                    last_state[offset] = 1;
                    last_rising[offset] = event.timestamp();
                } else if value == 0 && last_state[offset] == 1 {
                    //falling update current state and store duration of keypress
                    last_state[offset] = 0;
                    let held_for = event.timestamp() - last_rising[offset];
                    key_presses.push((offset, held_for));
                }
            }
        }
    }
    key_presses
}

fn configure_watching(
    chip: &mut Chip,
    offsets: &[u32],
) -> Result<(Vec<LineEventHandle>, Vec<PollFd>), Error> {
    // maps to the driver for the SoC (builtin) GPIO controller.
    let evt_handles = offsets
        .iter()
        .map(|off| chip.get_line(*off).unwrap())
        .map(|line| {
            line.events(
                LineRequestFlags::INPUT,
                EventRequestFlags::BOTH_EDGES,
                "sensor_central",
            )
            .unwrap()
        })
        .collect::<Vec<_>>();

    let pollables = evt_handles
        .iter()
        .map(|h| PollFd::new(h.as_raw_fd(), PollFlags::POLLIN | PollFlags::POLLPRI))
        .collect::<Vec<_>>();

    Ok((evt_handles, pollables))
}

pub fn start_monitoring(s: Sender<SensorValue>) -> Result<(), Error> {
    if let Some(mut chip) = gpio_cdev::chips()?
        .filter_map(Result::ok)
        .filter(|c| c.label() == "pinctrl-bcm2835")
        .next()
    {
        detect_and_handle(&mut chip, s)?;
        Ok(())
    } else {
        error!("could not find gpio chip");
        Err(Error::GPIONotFound)
    }
}
