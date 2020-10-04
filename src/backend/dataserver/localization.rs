use bitspec::{Field, FloatField};
use crate::SensorValue;

pub struct DataSet {
    pub set_id: u16,
    pub fields: &'static [Field],
    pub from: &'static [SensorValue],
}

pub const DATASETS: &'static [DataSet] = &[
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
];
