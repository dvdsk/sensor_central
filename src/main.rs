#![feature(drain_filter)]

use crossbeam_channel::bounded;
use serde::{Deserialize, Serialize};
use structopt::StructOpt;

mod backend;
mod buttons;
mod error;
mod sensors;

#[derive(StructOpt)]
#[structopt(name = "sensor_central")]
struct Opt {
    /// dataserver port
    #[structopt(short = "p", long = "port", default_value = "38972")]
    port: u16,
    /// full domain including any possible www prefix
    #[structopt(short = "d", long = "domain")]
    domain: String,
    #[structopt(short = "k", long = "dataserver-key")]
    dataserver_key: u64,
    /// home automation key
    #[structopt(short = "h", long = "ha-key")]
    ha_key: String,
    /// home automation domain
    #[structopt(short = "a", long = "ha-domain")]
    ha_domain: String,
    /// home automation port
    #[structopt(short = "o", long = "ha-port")]
    ha_port: u16,
    /// ble authentication key
    /// for example "[1,2,3,4]". If the key is shorter then 16 bytes it is
    /// padded with zeros
    #[cfg(feature = "ble")]
    #[structopt(long = "ble-key")]
    ble_key: sensors::ble::Key,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, Eq, PartialEq, Hash)]
pub enum Sensor {
    Temperature,
    Humidity,
    Pressure,
    TestSine,
    TestTriangle,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SensorValue {
    #[cfg(feature = "local")]
    ButtonPress(buttons::Press),
    Float(Sensor, f32),
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let _ = setup_logger().unwrap();

    let dataserver_url = format!("https://{}:{}/post_data", opt.domain, opt.port);
    let ha_url = format!("https://{}:{}/{}", opt.ha_domain, opt.ha_port, opt.ha_key);

    let mut dataserver = backend::Dataserver::new(opt.dataserver_key, dataserver_url);
    let home_automation = backend::HomeAutomation::new(ha_url);
    let (s, r) = bounded(10);

    #[cfg(feature = "local")]
    buttons::start_monitoring(s.clone()).unwrap();
    #[cfg(feature = "local")]
    sensors::local::start_monitoring(s.clone());
    #[cfg(feature = "ble")]
    sensors::ble::start_monitoring(s, opt.ble_key);

    loop {
        let data = r.recv().unwrap();

        let f1 = home_automation.handle(&data);
        let f2 = dataserver.handle(&data);
        let (res1, res2) = tokio::join!(f1, f2);

        backend::Dataserver::log_any_error(res1);
        backend::HomeAutomation::log_any_error(res2);
    }
}

fn setup_logger() -> Result<(), fern::InitError> {
    fern::Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "{}[{}][{}] {}",
                chrono::Local::now().format("[%H:%M:%S]"),
                record.target(),
                record.level(),
                message
            ))
        })
        .level(log::LevelFilter::Debug)
        .level_for("hyper", log::LevelFilter::Info)
        .level_for("reqwest", log::LevelFilter::Warn)
        .level_for("rustls", log::LevelFilter::Warn)
        .level_for("h2", log::LevelFilter::Warn)
        .chain(std::io::stdout())
        .apply()?;
    Ok(())
}
