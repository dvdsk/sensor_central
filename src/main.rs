#![feature(drain_filter)]

use crossbeam_channel::bounded;
use serde::{Deserialize, Serialize};
use simplelog::{Config, LevelFilter, SimpleLogger};
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
    #[structopt(short = "i", long = "node-id", default_value = "3")]
    node_id: u16,
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
    // ble authentication key
    #[structopt(long = "ble-key")]
    ble_key: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum Sensor {
    Temperature,
    Humidity,
    Pressure,
    Test,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SensorValue {
    ButtonPress(buttons::Press),
    Float(Sensor, f32),
}

#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let dataserver_url = format!("https://{}:{}/post_data", opt.domain, opt.port);
    let ha_url = format!("https://{}:{}/{}", 
        opt.ha_domain, opt.ha_port, opt.ha_key);

    let mut dataserver = backend::Dataserver::new(opt.node_id, opt.dataserver_key, dataserver_url);
    let home_automation = backend::HomeAutomation::new(ha_url);
    let (s, r) = bounded(10);

    buttons::start_monitoring(s.clone()).unwrap();
    sensors::start_monitoring(s, opt.ble_key.clone());

    loop {
        let data = r.recv().unwrap();

        let f1 = home_automation.handle(&data);
        let f2 = dataserver.handle(&data);
        let (res1, res2) = tokio::join!(f1, f2);

        backend::Dataserver::log_any_error(res1);
        backend::HomeAutomation::log_any_error(res2);
    }
}
