use std::collections::HashMap;
use std::mem::{discriminant, Discriminant};

use crate::SensorValue;
use bitspec::{Field, FieldValue};
use byteorder::{ByteOrder, LittleEndian};
use log::error;
use reqwest::{self, Client};

mod localization;

#[cfg(feature = "local")]
use crate::sensors::local;

#[derive(Default)]
struct Line {
    id: u16,
    key: u64,
    values: Vec<Option<FieldValue>>,
    fields: Vec<Field>,
}

impl Line {
    fn new(id: u16, key: u64, fields: impl Into<Vec<Field>>) -> Line {
        let fields = fields.into();
        Line {
            id,
            key,
            values: vec![None; fields.len()],
            fields,
        }
    }

    fn is_complete(&self) -> bool {
        self.values.iter().fold(true, |acc, x| acc && x.is_some())
    }

    fn encode(&mut self) -> Vec<u8> {
        let len: usize = self.fields.iter().map(|x| x.len() as usize).sum::<usize>();
        let mut line = vec![0u8; 10 + (len + 8 - 1) / 8];

        LittleEndian::write_u16(&mut line[0..2], self.id);
        LittleEndian::write_u64(&mut line[2..10], self.key);

        for (field, value) in self.fields.iter().zip(&mut self.values) {
            field.encode(value.take().unwrap(), &mut line[10..]);
        }
        line
    }
}

type LineIdx = usize;
type FieldIdx = usize;
pub struct Dataserver {
    client: Client,
    url: String,
    lines: Vec<Line>,
    to_line: HashMap<Discriminant<SensorValue>, (LineIdx, FieldIdx)>,
}

impl Dataserver {
    pub fn new(key: u64, url: String) -> Dataserver {
        let mut lines = Vec::new();
        let mut to_line = HashMap::new();

        #[cfg(feature = "ble")]
        for set in localization::DATASETS.iter() {
            let line = Line::new(set.set_id, key, set.fields);
            lines.push(line);
            let line_idx = lines.len() - 1;
            for (i, value) in set.from.iter().enumerate() {
                to_line.insert(discriminant(value), (line_idx, i));
            }
        }

        #[cfg(feature = "local")]
        {
            //add the local sensors manually
            let line_idx = lines.len();
            lines.push(Line::new(local::SET_ID, key, local::FIELDS));
            to_line.insert(discriminant(&SensorValue::Temperature(0.)), (line_idx, 0));
            to_line.insert(discriminant(&SensorValue::Humidity(0.)), (line_idx, 1));
            to_line.insert(discriminant(&SensorValue::Pressure(0.)), (line_idx, 2));
        }

        Self {
            client: Client::new(),
            url,
            lines,
            to_line,
        }
    }

    pub async fn handle(&mut self, value: &SensorValue) -> Result<(), reqwest::Error> {
        let (line_idx, field_idx) = self.to_line.get_mut(&discriminant(&value)).unwrap();
        let line = self.lines.get_mut(*line_idx as usize).unwrap();
        line.values[*field_idx as usize] = Some(FieldValue::from(*value));

        if line.is_complete() {
            let encoded = line.encode();
            self.client.post(&self.url).body(encoded).send().await?;
        }

        Ok(())
    }

    pub fn log_any_error(res: Result<(), reqwest::Error>) {
        if let Err(e) = res {
            error!("error during sending: {}", e);
        }
    }
}

impl From<SensorValue> for FieldValue {
    fn from(v: SensorValue) -> Self {
        use SensorValue::*;
        use FieldValue::*;

        match v {
            ButtonPress(p) => {
                unimplemented!();
            },
            Temperature(v) => F32(v),
            Humidity(v) => F32(v),
            Pressure(v) => F32(v),
            TestSine(v) => F32(v),
            TestTriangle(v) => F32(v),
        }
    }
}
