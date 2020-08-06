#![cfg(feature = "local")]

use gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use log::error;
use nix::poll::{poll, PollFd, PollFlags};
use smallvec::SmallVec;
use std::os::unix::io::AsRawFd;

use crate::error::Error;
use crate::SensorValue;
use crossbeam_channel::Sender;
use serde::{Deserialize, Serialize};
use std::thread;

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Button {
    LampLeft,
    LampMid,
    LampRight,

    DeskLeftMost,
    DeskLeft,
    DeskRight,
    DeskRightMost,

    DeskTop,
    DeskMid,
    DeskBottom,
}

impl Button {
    fn from_offset(offset: usize) -> Option<Self> {
        match offset {
            16 => Some(Self::LampLeft),
            12 => Some(Self::LampMid),
            13 => Some(Self::LampRight),

            //buttons on desk
            27 => Some(Self::DeskTop),
            22 => Some(Self::DeskMid),
            18 => Some(Self::DeskBottom),

            23 => Some(Self::DeskLeftMost),
            24 => Some(Self::DeskLeft),
            26 => Some(Self::DeskRight),
            17 => Some(Self::DeskRightMost),

            _ => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum Press {
    Long(Button),
    Short(Button),
}

impl Press {
    fn from(offset: usize, duration: u64) -> Option<Press> {
        let button = Button::from_offset(offset)?;
        if duration > MAX_TAP_LEN {
            Some(Press::Long(button))
        } else {
            Some(Press::Short(button))
        }
    }
}

// when pressing 22, 24 is often activated
// 23 works perfectly
// 24 also works perfectly
const MILLIS: u64 = 1_000_000; //nano to milli
const MAX_TAP_LEN: u64 = 600 * MILLIS;
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
                        if let Some(p) = Press::from(offset, down_duration) {
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
