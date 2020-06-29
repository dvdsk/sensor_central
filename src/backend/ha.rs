use crate::SensorValue;
use log::error;
use reqwest::{self, Client};

pub struct HomeAutomation {
    client: Client,
    url: String,
}

impl HomeAutomation {
    pub fn new(url: String) -> HomeAutomation {
        HomeAutomation {
            client: Client::new(),
            url,
        }
    }

    pub async fn handle(&self, data: &SensorValue) -> Result<(), reqwest::Error> {
        let encoded: Vec<u8> = bincode::serialize(&data).unwrap();
        self.client.post(&self.url).body(encoded).send().await?;
        Ok(())
    }

    pub fn log_any_error(res: Result<(), reqwest::Error>) {
        if let Err(e) = res {
            error!("error during sending: {}", e);
        }
    }
}
