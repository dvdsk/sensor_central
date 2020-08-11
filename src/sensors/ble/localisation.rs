use crate::{Sensor, SensorValue};
use bitspec::Field;
use crossbeam_channel::Sender;

#[derive(Clone, Debug)]
pub struct DeviceInfo {
    pub adress: &'static str,
    pub values: &'static [UuidInfo],
}

#[derive(Clone, Debug)]
pub struct UuidInfo {
    pub uuid: &'static str,
    pub set_id: u16,
    pub fields: &'static [Field<f32>],
    pub ha_values: &'static [Sensor],
}

impl PartialEq for UuidInfo {
    fn eq(&self, other: &UuidInfo) -> bool {
        self.uuid == other.uuid
    }
}

impl UuidInfo {
    pub fn process(&self, buffer: &[u8], s: &mut Sender<SensorValue>) {
        for (field, sensor) in self.fields.iter().zip(self.ha_values) {
            let value = field.decode::<f32>(buffer);
            let value = SensorValue::Float(*sensor, value);
            s.send(value).unwrap();
        }
    }

    pub fn byte_len(&self) -> usize {
        let last = self.fields.last().unwrap();
        let len = (last.offset + last.length + (8 - 1)) / 8;
        len as usize
    }
}

pub const SENSORS: &'static [DeviceInfo] = &[
    DeviceInfo {
        adress: "0A:0A:0A:0A:0A:0A",
        values: &[UuidInfo {
            set_id: 1,
            uuid: "93700001-1bb7-1599-985b-f5e7dc991483",
            fields: &[
                // Ble_reliability_testing_dataset
                Field::<f32> {
                    // Sine
                    decode_add: -5000.0000000000,
                    decode_scale: 1.0000000000,
                    length: 14,
                    offset: 0,
                },
                Field::<f32> {
                    // Triangle
                    decode_add: -10.0000000000,
                    decode_scale: 0.0500000000,
                    length: 10,
                    offset: 14,
                },
            ],
            ha_values: &[Sensor::TestSine, Sensor::TestTriangle],
        }],
    },
    /*DeviceInfo {
        adress: "0A:0A:0A:0A:0A:0A",
        values: &[
            UuidInfo {
                uuid: "93700002-1bb7-1599-985b-f5e7dc991483",
                fields: &[Field::<f32> {
                    decode_add: 1.,
                    decode_scale: 1.,
                    length: 1,
                    offset: 1,
                }],
                ha_values: &[Sensor::Humidity],
            }
        ]
    },*/
];
