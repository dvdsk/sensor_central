use gpio_cdev::{Chip, EventRequestFlags, LineEventHandle, LineRequestFlags};
use nix::poll::{poll, PollFd, PollFlags};
use protocol::{Press, Reading};
use std::os::unix::io::AsRawFd;
use std::sync::mpsc::Sender;
use tracing::error;

use std::thread;

use protocol::downcast_err::RpiButtonError as Error;
use protocol::large_bedroom::desk::Button;

fn button_from_offset(offset: usize, duration: u64) -> Option<Button> {
    let dur = duration as u16;
    match offset {
        // 16 => Some(LampLeft),
        // 12 => Some(LampMid),
        // 13 => Some(LampRight),

        //buttons on desk
        27 => Some(Button::OneOfThree(Press(dur))),
        22 => Some(Button::TwoOfThree(Press(dur))),
        18 => Some(Button::ThreeOfThree(Press(dur))),

        23 => Some(Button::FourOfFour(Press(dur))),
        24 => Some(Button::ThreeOfFour(Press(dur))),
        26 => Some(Button::TwoOfFour(Press(dur))),
        17 => Some(Button::OneOfFour(Press(dur))),

        _ => None,
    }
}

// when pressing 22, 24 is often activated
// 23 works perfectly
// 24 also works perfectly
const MILLIS: u64 = 1_000_000; //nano to milli
const N_LINES: usize = 54;
fn detect_and_handle(
    chip: &mut Chip,
    s: Sender<Result<Reading, protocol::Error>>,
) -> Result<(), Error> {
    use protocol::large_bedroom::desk::Error::Running;
    use protocol::large_bedroom::desk::{Reading, SensorError};
    use protocol::large_bedroom::{Error::Desk as DeskE, Reading::Desk as DeskR};
    use protocol::{Error::LargeBedroom as LbE, Reading::LargeBedroom as LbR};

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
                let key_presses = match key_presses {
                    Ok(k) => k,
                    Err(e) => {
                        let error = LbE(DeskE(Running(SensorError::Gpio(e))));
                        s.send(Err(error)).unwrap();
                        continue;
                    }
                };

                for (offset, down_duration) in key_presses {
                    if down_duration > 10 * MILLIS {
                        //debounce
                        if let Some(p) = button_from_offset(offset, down_duration) {
                            let reading = LbR(DeskR(Reading::Button(p)));
                            s.send(Ok(reading)).unwrap();
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
) -> Result<Vec<(usize, u64)>, Error> {
    let mut key_presses = Vec::new();

    for (_, handle) in pollables
        .iter()
        .zip(evt_handles.iter_mut())
        .filter_map(|(poll_fd, handles)| {
            poll_fd
                .revents()
                .and_then(|poll_res| Some((poll_res, handles)))
        })
        .filter(|(p, _)| p.contains(PollFlags::POLLIN))
    {
        let value = handle.get_value().map_err(|e| {
            error!("Could not get event value: {e}");
            Error::GetEventValue
        })?;
        let event = handle.get_event().map_err(|e| {
            error!("Could not get event value: {e}");
            Error::GetEventValue
        })?;

        let offset = handle.line().offset() as usize;

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

    Ok(key_presses)
}

fn configure_watching(
    chip: &mut Chip,
    offsets: &[u32],
) -> Result<(Vec<LineEventHandle>, Vec<PollFd>), Error> {
    // maps to the driver for the SoC (builtin) GPIO controller.
    let evt_handles: Result<Vec<_>, _> = offsets
        .iter()
        .map(|off| {
            let line = chip.get_line(*off).map_err(|e| {
                tracing::error!("Could not get line at offset: {off}, error: {e}");
                Error::GettingLine(*off)
            })?;

            line.events(
                LineRequestFlags::INPUT,
                EventRequestFlags::BOTH_EDGES,
                "sensor_central",
            )
            .map_err(|e| {
                tracing::error!("Could not get line event handle, error: {e}");
                Error::GettingLine(*off)
            })
        })
        .collect();

    let evt_handles = evt_handles?;
    let pollables = evt_handles
        .iter()
        .map(|h| PollFd::new(h.as_raw_fd(), PollFlags::POLLIN | PollFlags::POLLPRI))
        .collect::<Vec<_>>();

    Ok((evt_handles, pollables))
}

pub fn start_monitoring(tx: Sender<Result<Reading, protocol::Error>>) -> Result<(), Error> {
    let chips = match gpio_cdev::chips() {
        Ok(chips) => chips,
        Err(e) => {
            tracing::error!("error while listing chips: {}", e);
            return Err(Error::GpioChipNotFound);
        }
    };

    if let Some(mut chip) = chips
        .filter_map(Result::ok)
        .filter(|c| c.label() == "pinctrl-bcm2835")
        .next()
    {
        detect_and_handle(&mut chip, tx)?;
        Ok(())
    } else {
        tracing::error!("could not find gpio chip");
        Err(Error::GpioChipNotFound)
    }
}
