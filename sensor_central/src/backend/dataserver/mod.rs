use std::collections::HashMap;

use crate::SensorValue;
use bitspec::{Field, FieldValue};
use byteorder::{ByteOrder, LittleEndian};
use log::error;
use reqwest::{self, Client};

mod localization;

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
    to_line: HashMap<u64, (LineIdx, FieldIdx)>,
}

impl Dataserver {
    pub fn new(key: u64, url: String) -> Dataserver {
        let mut lines = Vec::new();
        let mut to_line = HashMap::new();

        for set in localization::DATASETS.iter() {
            let line = Line::new(set.set_id, key, set.fields);
            lines.push(line);
            let line_idx = lines.len() - 1;
            for (i, value) in set.from.iter().enumerate() {
                dbg!(&value);
                dbg!(&value.to_key());
                to_line.insert(value.to_key(), (line_idx, i));
            }
        }

        Self {
            client: Client::new(),
            url,
            lines,
            to_line,
        }
    }

    pub async fn handle(&mut self, value: &SensorValue) -> Result<(), reqwest::Error> {
        if let Some((line_idx, field_idx)) = self.to_line.get_mut(&value.to_key()) {
            // dbg!(&value.to_key());
            let line = self.lines.get_mut(*line_idx as usize).unwrap();
            line.values[*field_idx as usize] = Some(to_fieldval(*value));

            if line.is_complete() {
                let encoded = line.encode();
                self.client.post(&self.url).body(encoded).send().await?;
            }
        }
        Ok(())
    }

    pub fn log_any_error(res: Result<(), reqwest::Error>) {
        if let Err(e) = res {
            error!("error during sending: {}", e);
        }
    }
}

fn to_fieldval(v: SensorValue) -> FieldValue {
    use SensorValue::*;
    use FieldValue::*;

    match v {
        ButtonPress(p) => {
            // how do we store this a dataset?
            // need 4 bits...
            unimplemented!();
        },
        Temperature(v) => F32(v),
        Humidity(v) => F32(v),
        Pressure(v) => F32(v),
        TestSine(v) => F32(v),
        TestTriangle(v) => F32(v),
    }
}
