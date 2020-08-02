use bitspec::Field;

pub struct DeviceInfo {
    pub adress: &'static str,
    pub values: &'static [UuidInfo],
}

pub struct UuidInfo {
    pub uuid: &'static str,
    fields: &'static [Field<f32>],
    pub ha_values: &'static [SensorValue],  
}

impl UuidInfo {
    pub fn process(&self, buffer: &[u8], s: Sender<SensorValue>) {
        for (field, sensorvalue) in self.fields.iter().zip(self.ha_values) {
            let value = field.decode::<f32>(buffer);
            let value = package(value, sensorvalue);
            s.send(value).unwrap();
        }
    }

    pub fn byte_len(&self) -> usize {
        let last = self.fields.last().unwrap();
        let len = (last.offset + last.length + (8-1)) /8;
        len as usize
    }
}

pub const SENSORS: &'static [DeviceInfo] = &[
    DeviceInfo {
        adress: "0A:0A:0A:0A:0A:0A",
        values: &[
            UuidInfo {
                uuid: "93700001-1bb7-1599-985b-f5e7dc991483",
                fields: &[Field::<f32> {
                    decode_add: 1.,
                    decode_scale: 1.,
                    length: 1,
                    offset: 1,
                }],
                ha_values: &[SensorValue::Humidity(0.)],
            }
        ]
    },
    DeviceInfo {
        adress: "0A:0A:0A:0A:0A:0A",
        values: &[
            UuidInfo {
                uuid: "93700001-1bb7-1599-985b-f5e7dc991483",
                fields: &[Field::<f32> {
                    decode_add: 1.,
                    decode_scale: 1.,
                    length: 1,
                    offset: 1,
                }],
                ha_values: &[SensorValue::Humidity(0.)],
            }
        ]
    },
];

fn package(value: f32, sensorvalue: &SensorValue) -> SensorValue {
    match sensorvalue {
        SensorValue::ButtonPress(_) => panic!("not a remote value"),
        SensorValue::Humidity(_) => panic!("not a remote value"),
        SensorValue::Temperature(_) => panic!("not a remote value"),
        SensorValue::Pressure(_) => panic!("not a remote value"),
        
        SensorValue::Test(_) => SensorValue::Test(value),
    }
}