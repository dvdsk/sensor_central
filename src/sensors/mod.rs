use crossbeam_channel::Sender;
use std::thread;
use std::time::Duration;

mod local;
mod ble;
use super::SensorValue;

pub fn start_monitoring(s: Sender<SensorValue>) {
    thread::spawn(move || {
        let mut local_sensors = local::init();
        loop {
            //get all measurements
            let (hum, temp, pressure) = local::measure_and_record(&mut local_sensors);
            s.send(SensorValue::Temperature(temp)).unwrap();
            s.send(SensorValue::Humidity(hum)).unwrap();
            s.send(SensorValue::Pressure(pressure)).unwrap();

            std::thread::sleep(Duration::from_secs(5));
        }
    });
}
