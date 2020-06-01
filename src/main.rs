use byteorder::{LittleEndian, ByteOrder};
use structopt::StructOpt;
use log::error;
use tokio::time::delay_for;
use simplelog::{SimpleLogger, LevelFilter, Config};

mod local;
mod fields;

use std::time::Duration;
use local::{TEMPERATURE, HUMIDITY, PRESSURE, STOP_ENCODE};

#[derive(StructOpt)]
#[structopt(name = "sensor_central")]
struct Opt {
    #[structopt(short = "p", long = "port", default_value = "38972")]
	port: u16,
	// full domain including any possible www prefix
    #[structopt(short = "d", long = "domain")]
	domain: String,

    #[structopt(short = "n", long = "node-id", default_value= "3")]
	node_id: u16,
	
    #[structopt(short = "s", long = "dataserver-key")]
	dataserver_key: u64,
}

#[tokio::main]
async fn main() {

	let opt = Opt::from_args();
    let sample_period = Duration::from_secs(5);
    let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    let post_url = format!("https://{}:{}/post_data", opt.domain, opt.port);

    let mut local_sensors = local::init();
    loop {
        //get all measurements
        let (hum, temp, pressure) = local::measure_and_record(&mut local_sensors);

        //encode all data
        let mut line = vec!(0u8; (STOP_ENCODE+8-1)/8);
        LittleEndian::write_u16(&mut line[0..2], opt.node_id);
        LittleEndian::write_u64(&mut line[2..10], opt.dataserver_key);

        TEMPERATURE.encode::<f32>(temp, &mut line);
        HUMIDITY.encode::<f32>(hum, &mut line);
        PRESSURE.encode::<f32>(pressure, &mut line);

        let client = reqwest::Client::new();
        let send = client.post(&post_url)
            .body(line)
            .timeout(sample_period)
            .send();
        let sleep = delay_for(sample_period);
        let (_,res) = tokio::join!(sleep, send);
        if let Err(e) = res {
            error!("could not send data to dataserver, error: {:?}", e);
        }
        
    }
}
