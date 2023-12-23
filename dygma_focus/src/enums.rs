use dygma_focus_proc_macros::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, NumStrEnum)]
pub enum LedMode {
    PerLayer,
    RainbowWave,
    RainbowSingle,
    Stalker,
    Red,
    Green,
    Blue,
    White,
    Off,
    GreenInner,
    Bluetooth,
}

#[derive(Debug, Copy, Clone, PartialEq, Eq, Serialize, Deserialize, NumStrEnum)]
pub enum WirelessPowerMode {
    Low,
    Medium,
    High,
}
