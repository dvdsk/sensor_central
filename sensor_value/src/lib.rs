use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{Deserialize, Serialize};
use bitspec::FieldValue;

#[derive(Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub enum Button {
    LampLeft,
    LampMid,
    LampRight,

    DeskLeftMost,
    DeskLeft,
    DeskRight,
    DeskRightMost,

    DeskTop,
    DeskMid,
    DeskBottom,
}

#[derive(Clone, Copy, Debug, Hash, Serialize, Deserialize)]
pub struct Press {
    pub button: Button,
    pub duration: u16, //0-10 seconds can be stored
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum SensorValue {
    ButtonPress(Press),
    Temperature(f32),
    Humidity(f32),
    Pressure(f32),
    TestSine(f32),
    TestTriangle(f32),
    TestSine2(f32),
    TestTriangle2(f32),
    TestSine3(f32),
    TestTriangle3(f32),
    TestButtonOne(f32),
    MovementSensor(bool),
}

impl SensorValue {
    /// Every variant containing a value (f32) gets the same
    /// key regardless of the current value
    pub fn to_key(&self) -> u64 {
        let mut hasher = DefaultHasher::new();
        std::mem::discriminant(self).hash(&mut hasher);
        match self {
            SensorValue::ButtonPress(p) => p.hash(&mut hasher),
            _ => (),
        }
        hasher.finish()
    }
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
            (F32(v), TestSine2(_)) => TestSine2(v),
            (F32(v), TestTriangle2(_)) => TestTriangle2(v),
            _ => panic!("unhandled"),
        }
    }
}
