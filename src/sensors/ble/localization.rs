use crate::SensorValue;
use bitspec::{Field, FieldValue, FloatField};
use crossbeam_channel::Sender;

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub adress: &'static str,
    pub values: &'static [UuidInfo],
}

#[derive(Clone, Debug)]
pub enum UuidInfo {
    Static(Info),
    Dynamic(Info),
}

#[derive(Clone, Debug)]
pub struct Info {
    pub uuid: &'static str,
    pub fields: &'static [Field],
    pub sensorval: &'static [SensorValue],
}

impl PartialEq for UuidInfo {
    fn eq(&self, other: &UuidInfo) -> bool {
        self.uuid() == other.uuid()
    }
}

impl UuidInfo {
    pub fn process(&self, buffer: &[u8], s: &mut Sender<SensorValue>) {
        match &self {
            Self::Static(info) => process_static(info, buffer, s),
            Self::Dynamic(info) => process_dynamic(info, buffer, s),
        }
    }
    pub fn byte_len(&self) -> usize {
        match &self {
            Self::Static(info) => {
                let last = info.fields.last().unwrap();
                let len = (last.offset() + last.len() + (8 - 1)) / 8;
                len as usize
            }
            Self::Dynamic(info) => todo!(),
        }
    }
    pub fn uuid(&self) -> &str {
        match &self {
            Self::Static(i) => i.uuid,
            Self::Dynamic(i) => i.uuid,
        }
    }
}

pub fn process_static(info: &Info, buffer: &[u8], s :&mut Sender<SensorValue>){
    for (field, sensor) in info.fields.iter().zip(info.sensorval) {
        let value = field.decode(buffer);
        let value = SensorValue::from((value,*sensor));
        s.send(value).unwrap();
    }
}

pub fn process_dynamic(info: &Info, buffer: &[u8], s :&mut Sender<SensorValue>){
    todo!()
}

impl From<(FieldValue, SensorValue)> for SensorValue {
    fn from(t: (FieldValue, SensorValue)) -> Self {
        use FieldValue::*;
        use SensorValue::*;

        match t {
            (F32(v), Temperature(_)) => Temperature(v),
            (F32(v), Humidity(_)) => Humidity(v),
            (F32(v), Pressure(_)) => Pressure(v),
            (F32(v), TestSine(_)) => TestSine(v),
            (F32(v), TestTriangle(_)) => TestTriangle(v),
            _ => panic!("unhandled"),
        }
    }
}

pub const SENSORS: &'static [DeviceInfo] = &[
    DeviceInfo {
        adress: "0A:0A:0A:0A:0A:0A",
        values: &[
            UuidInfo::Static(Info {
                uuid: "93700001-1bb7-1599-985b-f5e7dc991483",
                fields: &[
                    // Ble_reliability_testing_dataset
                    Field::F32(FloatField {
                        // Sine
                        decode_add: -5000.0000000000,
                        decode_scale: 1.0000000000,
                        length: 14,
                        offset: 0,
                    }),
                    Field::F32(FloatField {
                        // Triangle
                        decode_add: -10.0000000000,
                        decode_scale: 0.0500000000,
                        length: 10,
                        offset: 14,
                    }),
                ],
                sensorval: &[SensorValue::TestSine(0.), SensorValue::TestTriangle(0.)],
            }),
            // UuidInfo::Dynamic(DynamicInfo {
            
            // }),
        ],
    },
];