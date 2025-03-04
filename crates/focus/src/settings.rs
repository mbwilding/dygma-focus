use crate::color::*;
use crate::enums::{LedMode, WirelessPowerMode};

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "serde_camel_case", serde(rename_all = "camelCase"))]
pub struct Settings {
    pub keymap_custom: Vec<u16>,
    pub keymap_default: Vec<u16>,
    pub keymap_only_custom: bool,
    pub settings_default_layer: u8,
    pub superkeys_map: Vec<u16>,
    pub superkeys_wait_for: u16,
    pub superkeys_timeout: u16,
    pub superkeys_repeat: u16,
    pub superkeys_hold_start: u16,
    pub superkeys_overlap: u8,
    pub led_mode: LedMode,
    pub led_brightness_keys_wired: u8,
    pub led_brightness_underglow_wired: Option<u8>,
    pub led_brightness_keys_wireless: Option<u8>,
    pub led_brightness_underglow_wireless: Option<u8>,
    pub led_fade: Option<u16>,
    pub led_theme: Vec<RGB>,
    pub palette_rgb: Option<Vec<RGB>>,
    pub palette_rgbw: Option<Vec<RGBW>>,
    pub color_map: Vec<u8>,
    pub led_idle_true_sleep: Option<bool>,
    pub led_idle_true_sleep_time: Option<u16>,
    pub led_idle_time_limit_wired: u16,
    pub led_idle_time_limit_wireless: Option<u16>,
    pub qukeys_hold_timeout: u16,
    pub qukeys_overlap_threshold: u16,
    pub macros_map: Vec<u8>,
    pub mouse_speed: u8,
    pub mouse_delay: u16,
    pub mouse_acceleration_speed: u8,
    pub mouse_acceleration_delay: u16,
    pub mouse_wheel_speed: u8,
    pub mouse_wheel_delay: u16,
    pub mouse_speed_limit: u8,
    pub wireless_battery_saving_mode: Option<bool>,
    pub wireless_rf_power_level: Option<WirelessPowerMode>,
    pub wireless_rf_channel_hop: Option<bool>,
}
