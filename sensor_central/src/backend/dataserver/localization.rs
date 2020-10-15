use bitspec::{Field, FloatField};
use crate::SensorValue;

pub struct DataSet {
    pub set_id: u16,
    pub fields: &'static [Field],
    pub from: &'static [SensorValue],
}

/// use spec generator to generate this from a template in the templates dir
pub const DATASETS: &'static [DataSet] = &[
    DataSet {
        set_id: 5,
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
        from: &[SensorValue::TestSine2(0.), SensorValue::TestTriangle2(0.)],
    },
    DataSet {
        set_id: 4,
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
        from: &[SensorValue::TestSine(0.), SensorValue::TestTriangle(0.)],
    },
    DataSet {
        set_id: 3,
        fields: &[ // Desk_Sensors
            Field::F32(FloatField { // temperature
                decode_add: -20.0000000000,
                decode_scale: 0.0099999998,
                length: 13,
                offset: 0	}),
            Field::F32(FloatField { // humidity
                decode_add: 0.0000000000,
                decode_scale: 0.0080000004,
                length: 14,
                offset: 13	}),
            Field::F32(FloatField { // pressure
                decode_add: 30000.0000000000,
                decode_scale: 0.1800000072,
                length: 19,
                offset: 27	}),
        ],
        from: &[SensorValue::Temperature(0.), SensorValue::Humidity(0.), SensorValue::Pressure(0.)],
    },
];
