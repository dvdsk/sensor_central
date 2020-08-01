use std::time::Duration;
use std::fs::File;
use std::io::Read;
use std::sync::mpsc::Sender;
use std::os::unix::io::AsRawFd;

use bluebus::{BleBuilder, Ble};
use nix::poll::{poll, PollFd, PollFlags};

use crate::error::Error;
use crate::SensorValue;
use crate::backend::Field;

mod info;
use info::{DeviceInfo, SENSORS};

struct BleSensors<'a> {
    sensors: &'a[DeviceInfo],
    files: Vec<File>,
    ble: Ble,
}

fn has_io(pollable: &PollFd) -> bool {
    if let Some(poll_res) = pollable.revents() {
        if poll_res.contains(PollFlags::POLLIN) {
            return true;
        }
    }
    false
}

const PAIRING_TIMEOUT: Duration = Duration::from_secs(15);
impl<'a> BleSensors<'a> {
    pub fn new() -> Result<Self, Error> {
        let ble = BleBuilder::new().build()?;

        Ok(BleSensors{
            sensors: SENSORS,
            files: Vec::new(),
            ble,
        })
    }

    pub fn connect(&mut self) -> Result<(), Error> {
        for info in self.sensors.iter() {
            self.ble.connect(info.adress)?;
            let get_key = self.setup_key(info.adress)?;
            self.ble.pair(info.adress, get_key, PAIRING_TIMEOUT)?;
            
            let device_files = Vec::new();
            for uuid in info.values.iter().map(|u| u.uuid) {
                let file = self.ble.notify(info.adress, uuid)?;
                self.files.push(file);
            }
        }
        Ok(())
    }

    fn setup_key(&mut self, adress: &str) -> Result<impl Fn() -> u32, Error> {
        let get_key = || 123456;
        Ok(get_key)
    }

    fn handle(&mut self, s: Sender<SensorValue>) {
        let buffer = [0u8; 100];
        let mut infos = Vec::new();
        
        for device_info in self.sensors {
            for info in device_info.values {
                infos.push(info);
            }  
        }

        let pollables = self.files.iter()
            .map(|f| PollFd::new(f.as_raw_fd(), PollFlags::POLLIN))
            .collect::<Vec<_>>();
        
        loop {
            if poll(&mut pollables, -1).unwrap() < 1 {
                dbg!("poll() failed"); //TODO improve on this
            }

            for (i, pollable) in pollables.iter().enumerate() {
                if !has_io(pollable) {continue;}

                let info = infos[i];
                self.files[i].read(&mut &buffer[..info.byte_len()]).unwrap();
                info.process(&buffer, s);
            }
        }
    }
}
