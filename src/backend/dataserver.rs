use crate::SensorValue;
use byteorder::{ByteOrder, LittleEndian};
use log::error;
use reqwest::{self, Client};
use bitspec::Field;

#[derive(Default)]
struct Line {
    id: u16,
    key: u64,
    values: Vec<Option<f32>>,
    fields: Vec<Field<f32>>,
}

impl Line {
    fn new(id: u16, key: u64, fields: Vec<Field<f32>>) -> Line {
        Line {
            id,
            key,
            values: vec![None; fields.len()],
            fields,
        }
    }

    fn is_complete(&self) -> bool {
        self.values
            .iter()
            .fold(true, |acc, x| acc && x.is_some())
    }
    
    fn encode(&mut self) -> Vec<u8> {
        let len: usize = self.fields.iter().map(|x| x.length as usize).sum::<usize>();
        let mut line = vec![0u8; 10+(len + 8 - 1) / 8];

        LittleEndian::write_u16(&mut line[0..2], self.id);
        LittleEndian::write_u64(&mut line[2..10], self.key);

        for (field, value) in self.fields.iter().zip(&mut self.values) {
            field.encode::<f32>(value.unwrap(), &mut line);
            *value = None;
        }
        line
    }
}

pub struct Dataserver {
    client: Client,
    connected: Line,
    url: String,
    //remote; LIne
}

impl Dataserver {
    pub fn new(id: u16, key: u64, url: String) -> Dataserver {
        Dataserver {
            client: Client::new(),
            connected: Line::new(id, key, vec![TEMPERATURE, HUMIDITY, PRESSURE]),
            url,
        }
    }

    pub async fn handle(&mut self, data: &SensorValue) -> Result<(), reqwest::Error> {
        match data {
            SensorValue::Temperature(v) => self.connected.values[0] = Some(*v),
            SensorValue::Humidity(v) => self.connected.values[1] = Some(*v),
            SensorValue::Pressure(v) => self.connected.values[2] = Some(*v),
            _ => return Ok(()),
        }

        if self.connected.is_complete() {
            let line = self.connected.encode();
            self.client.post(&self.url).body(line).send().await?; //check this
        }

        Ok(())
    }

    pub fn log_any_error(res: Result<(), reqwest::Error>) {
        if let Err(e) = res {
            error!("error during sending: {}", e);
        }
    }
}

////////// List of fields //////////////
pub const TEMPERATURE: Field<f32> = Field {
    offset: 80, //bits
    length: 13, //bits (max 32 bit variables)

    decode_scale: 0.009999999776482582,
    decode_add: -20.0f32,
};
pub const HUMIDITY: Field<f32> = Field {
    offset: 80 + TEMPERATURE.length,
    length: 14,

    decode_scale: 0.00800000037997961,
    decode_add: 0.0,
};
pub const PRESSURE: Field<f32> = Field {
    offset: 80 + TEMPERATURE.length + HUMIDITY.length,
    length: 19,

    decode_scale: 0.18000000715255738,
    decode_add: 30000.0,
};
