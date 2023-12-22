use custom_macros::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, StrEnum)]
pub enum LedMode {
    Standard,
    Rainbow,
    Cycle,
    HeatMap,
    Red,
    Green,
    Blue,
    White,
    Off,
    GreenInner,
    Bluetooth,
}
