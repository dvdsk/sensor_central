#[cfg(feature = "ble")]

use std::time::{Duration, Instant};
use std::fs::{File};
use std::io::Read;
use std::thread;
use crossbeam_channel::Sender;
use std::os::unix::io::{RawFd, FromRawFd};

use bluebus::{BleBuilder, Ble};
use nix::poll::{poll, PollFd, PollFlags};

use aes::Aes128;
use aes::block_cipher::{BlockCipher, NewBlockCipher};
use aes::block_cipher::generic_array::GenericArray;
use rand::Rng;

use crate::error::Error;
use crate::SensorValue;

mod localisation;
use localisation::{DeviceInfo, UuidInfo, SENSORS};
mod error;
use error::ConnectionError;

const SEC_CHAR_UUID: &'static str = "93700004-1bb7-1599-985b-f5e7dc991483";

pub struct BleSensors {
    disconnected: Vec<DeviceInfo>,
    connected: Vec<DeviceInfo>,
    notify_streams: NotifyStreams,

    ble: Ble,
    key: [u8;16],
}

fn has_io(pollable: &PollFd) -> bool {
    if let Some(poll_res) = pollable.revents() {
        if poll_res.contains(PollFlags::POLLIN) {
            return true;
        }
    }
    false
}

#[derive(Default)]
struct NotifyStreams {
    pollables: Vec<PollFd>,
    files: Vec<File>,
    infos: Vec<UuidInfo>,
}

impl NotifyStreams {
    pub fn add(&mut self, fds: Vec<RawFd>, device: &DeviceInfo){
        
        self.pollables.extend(fds.iter()
            .map(|fd| PollFd::new(*fd, PollFlags::POLLIN)));
        self.files.extend(fds.iter()
            .map(|fd| unsafe {File::from_raw_fd(*fd)} ));
        //self.fds.append(fds);es;

        self.infos.extend(device.values.iter().cloned());
    }

    pub fn remove(&mut self, device: &DeviceInfo){
        let mut start = None;
        for (i,info) in self.infos.iter().enumerate() {
            if info == device.values.first().unwrap() {
                start = Some(i);
            }
        } 

        let start = start.expect("device should be in infos list!");
        for i in start..start+device.values.len() {
            self.pollables.remove(i);
            self.files.remove(i);
            self.infos.remove(i);
        }
    }

    pub fn handle(&mut self, buffer: &mut [u8], s: &mut Sender<SensorValue>){
        if poll(&mut self.pollables, -1).unwrap() < 1 {
            dbg!("poll() failed"); //TODO improve on this
        }

        for (i, pollable) in self.pollables.iter().enumerate() {
            if !has_io(pollable) {continue;}

            let info = &self.infos[i];
            self.files[i].read(&mut buffer[..info.byte_len()]).unwrap();
            info.process(&buffer, s);
        }
    }
}

const PAIRING_TIMEOUT: Duration = Duration::from_secs(15);
impl BleSensors {
    pub fn new(ble_key: String) -> Result<Self, Error> {
        let mut ble = BleBuilder::new().build()?;
        ble.start_discovery()?;
        thread::sleep(Duration::from_secs(5));
        ble.stop_discovery()?;

        let mut key = [0u8; 16];
        key[..usize::min(ble_key.len(),16)]
            .copy_from_slice(ble_key.as_str().as_bytes());

        dbg!(key);
        Ok(BleSensors{
            disconnected: SENSORS.to_vec(),
            connected: Vec::new(),
            
            notify_streams: NotifyStreams::default(),
            
            ble,
            key,
        })
    }

    pub fn reconnect_disconnected(&mut self) -> Result<(), ConnectionError> {
        let connected = &mut self.connected;
        let disconnected = &mut self.disconnected;
        let notify_streams = &mut self.notify_streams;
        let ble = &mut self.ble;
        let key = &self.key;

        disconnected.drain_filter(|device| {
            let res = Self::connect_device(ble, device, key);
            match res {
                Ok(fds) => {
                    connected.push(device.clone());
                    notify_streams.add(fds, device);
                    true //remove device from disconnected
                }
                Err(e) => {
                    if !e.is_recoverable() {
                        panic!("ran into unrecoverable error during connection of device: {}, error was: {:?}", device.adress, e);
                    }
                    false //keep device in disconnected
                }
            }
        });
        Ok(())
    }

    fn connect_device(ble: &mut Ble, device: &DeviceInfo, key: &[u8;16])
     -> Result<Vec<RawFd>, ConnectionError> {
    
        dbg!(&device.adress);
        ble.connect(device.adress)?;
        let get_key = Self::setup_key(ble, device.adress, key)?;
        ble.pair(device.adress, get_key, PAIRING_TIMEOUT)?;
        
        let test: Result<Vec<RawFd>, bluebus::Error> = device.values
            .iter()
            .map(|u| ble.notify(device.adress, u.uuid))
            .collect();
        let test = test?; //TODO cleanup
        Ok(test)
    }

    fn setup_key(ble: &mut Ble, adress: &str, key: &[u8;16]) -> Result<impl Fn() -> u32, bluebus::Error> {
        let mut rng = rand::thread_rng();
        let nonce: u32 = rng.gen_range(0, 999999);
        let mut nonce_array = [0u8; 16];
        nonce_array[..4].copy_from_slice(&nonce.to_be_bytes());
        dbg!(nonce_array);

        dbg!(nonce);
        let cipher = Aes128::new(GenericArray::from_slice(key));
        let mut block = GenericArray::from_mut_slice(&mut nonce_array);
        cipher.encrypt_block(&mut block);
        dbg!(nonce_array);

        ble.write(adress, SEC_CHAR_UUID, nonce_array)?;
        let get_key = move || nonce;
        Ok(get_key)
    }

    fn check_for_disconnects(&mut self) {
        let connected = &mut self.connected;
        let notify_streams = &mut self.notify_streams;
        let ble = &mut self.ble;

        connected.drain_filter(|device| {
            if ble.is_connected(device.adress).unwrap() {
                false
            } else {
                notify_streams.remove(device);
                true
            }
        });
    }
    
    pub fn handle(&mut self, mut s: Sender<SensorValue>) {
        const TIMEOUT: Duration = Duration::from_secs(5);
        let mut buffer = [0u8; 100];
        let now = Instant::now();

        loop {
            if now.elapsed() > TIMEOUT {
                self.check_for_disconnects();
            }

            self.reconnect_disconnected().unwrap();
            self.notify_streams.handle(&mut buffer, &mut s);
        }
    }
}
