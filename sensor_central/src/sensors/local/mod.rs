use bme280::{self, BME280};
use hal::{Delay, I2cdev};
use linux_embedded_hal as hal;

use crossbeam_channel::Sender;
use std::thread;
use std::time::Duration;

use crate::SensorValue;

pub fn init() -> BME280<I2cdev, Delay> {
    // using Linux I2C Bus #1 in this example
    let i2c_bus = I2cdev::new("/dev/i2c-1").unwrap();
    // initialize the BME280 using the primary I2C address 0x77
    let mut bme280 = BME280::new_primary(i2c_bus, Delay);
    // initialize the sensor
    bme280.init().unwrap();
    bme280
}

pub fn measure_and_record(bme: &mut BME280<I2cdev, Delay>) -> (f32, f32, f32) {
    // measure temperature, pressure, and humidity
    let measurements = bme.measure().unwrap();
    (
        measurements.humidity,
        measurements.temperature,
        measurements.pressure,
    )
}

pub fn start_monitoring(s: Sender<SensorValue>) {
    thread::spawn(move || {
        let mut local_sensors = init();
        loop {
            //get all measurements
            let (hum, temp, pressure) = measure_and_record(&mut local_sensors);
            s.send(SensorValue::Temperature(temp)).unwrap();
            s.send(SensorValue::Humidity(hum)).unwrap();
            s.send(SensorValue::Pressure(pressure)).unwrap();

            std::thread::sleep(Duration::from_secs(5));
        }
    });
}
