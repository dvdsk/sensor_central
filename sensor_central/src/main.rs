#![feature(drain_filter)]

use crossbeam_channel::bounded;
use structopt::StructOpt;
use sensor_value::SensorValue;

mod backend;
mod buttons;
mod error;
mod sensors;

#[derive(StructOpt)]
#[structopt(name = "sensor_central")]
struct Opt {
    /// dataserver port
    #[structopt(long = "data-port")]
    data_port: u16,
    /// use a remote data server instance, default is to use
    /// use 127.0.0.1
    #[structopt(long = "remote-data-server")]
    data_domain: Option<String>,
    /// dataserver key 
    #[structopt(long = "data-key")]
    dataserver_key: u64,
    /// home automation key
    #[structopt(long = "ha-key")]
    ha_key: String,
    /// use a remote home automation server instance, default is to use
    /// use 127.0.0.1
    #[structopt(long = "remote-ha-server")]
    ha_domain: Option<String>,
    /// home automation port
    #[structopt(long = "ha-port")]
    ha_port: u16,
    /// ble authentication key
    /// for example "[1,2,3,4]". If the key is shorter then 16 bytes it is
    /// padded with zeros
    #[structopt(long = "ble-key")]
    ble_key: sensors::ble::Key,
}


#[tokio::main]
async fn main() {
    let opt = Opt::from_args();
    let _ = setup_logger().unwrap();

    let dataserver_url = if let Some(domain) = opt.data_domain {
        format!("https://{}:{}/post_data", domain, opt.data_port)
    } else {
        format!("http://127.0.0.1:{}/post_data", opt.data_port)
    };

    let ha_url = if let Some(domain) = opt.ha_domain {
        format!("https://{}:{}/{}", domain, opt.ha_port, opt.ha_key)
    } else {
        format!("http://127.0.0.1:{}/{}", opt.ha_port, opt.ha_key)
    };

    let mut dataserver = backend::Dataserver::new(opt.dataserver_key, dataserver_url);
    let home_automation = backend::HomeAutomation::new(ha_url);
    let (s, r) = bounded(10);

    buttons::start_monitoring(s.clone()).unwrap();
    sensors::local::start_monitoring(s.clone());
    sensors::ble::start_monitoring(s, opt.ble_key);

    loop {
        let data: SensorValue = r.recv().unwrap();

        let f1 = home_automation.handle(&data);
        let f2 = dataserver.handle(&data);
        let (res1, res2) = tokio::join!(f1, f2);

        backend::HomeAutomation::log_any_error(res1);
        backend::Dataserver::log_any_error(res2);
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
