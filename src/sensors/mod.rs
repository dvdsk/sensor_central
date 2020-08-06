use crossbeam_channel::Sender;
use std::thread;
use std::time::Duration;

mod local;
mod ble;
use crate::{SensorValue, Sensor};

pub fn start_monitoring(s1: Sender<SensorValue>, ble_key: String) {
    let s2 = s1.clone();
    #[cfg(feature = "local")]
    thread::spawn(move || {
        let mut local_sensors = local::init();
        loop {
            //get all measurements
            let (hum, temp, pressure) = local::measure_and_record(&mut local_sensors);
            s2.send(SensorValue::Float(Sensor::Temperature, temp)).unwrap();
            s2.send(SensorValue::Float(Sensor::Humidity, hum)).unwrap();
            s2.send(SensorValue::Float(Sensor::Pressure, pressure)).unwrap();

            std::thread::sleep(Duration::from_secs(5));
        }
    });

    #[cfg(feature = "ble")]
    thread::spawn(move || {
        let mut sensors = ble::BleSensors::new(ble_key).unwrap();
        sensors.handle(s1);
    });
}
