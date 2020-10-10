use std::hash::{Hash, Hasher};
use std::collections::hash_map::DefaultHasher;
use serde::{Deserialize, Serialize};

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
