# Dygma Focus API (Rust)

[<img alt="crates.io" src="https://img.shields.io/crates/v/dygma_focus?style=for-the-badge&color=fc8d62&logo=rust" height="20">](https://crates.io/crates/dygma_focus)

## About

This crate is a Rust implementation of the Dygma Focus API.

Make sure to not have Bazecor running and connected while trying to communicate with your keyboard.

## Usage

Cargo.toml

```toml
[dependencies]
anyhow = "1.0"
dygma_focus = "0.3"
```

src/main.rs

```rust
use anyhow::Result;
use dygma_focus::prelude::*;

fn main() -> Result<()> {
    // Open the first device found and declare as mutable
    // Other constructors are under Focus::new_*
    let mut focus = Focus::new_first_available()?;

    // Here are some example get methods, most have a matching set method
    // There are also other methods for triggering macros or switching layers for example
    println!("version: {}", &focus.version_get()?);
    println!("keymap_custom: {:?}", &focus.keymap_custom_get()?);
    println!("keymap_default: {:?}", &focus.keymap_default_get()?);
    println!("keymap_only_custom: {}", &focus.keymap_only_custom_get()?);
    println!("settings_default_layer: {}", &focus.settings_default_layer_get()?);
    println!("settings_valid: {}", &focus.settings_valid_get()?);
    println!("settings_version: {}", &focus.settings_version_get()?);
    println!("settings_crc: {}", &focus.settings_crc_get()?);
    println!("eeprom_contents: {}", &focus.eeprom_contents_get()?);
    println!("eeprom_free: {}", &focus.eeprom_free_get()?);
    println!("superkeys_map: {:?}", &focus.superkeys_map_get()?);
    println!("superkeys_wait_for: {:?}", &focus.superkeys_wait_for_get()?);
    println!("superkeys_timeout: {:?}", &focus.superkeys_timeout_get()?);
    println!("superkeys_repeat: {:?}", &focus.superkeys_repeat_get()?);
    println!("superkeys_hold_start: {:?}", &focus.superkeys_hold_start_get()?);
    println!("superkeys_overlap: {}", &focus.superkeys_overlap_get()?);
    println!("led_at: {:?}", &focus.led_at_get(0)?);
    println!("led_mode: {:?}", &focus.led_mode_get()?);
    println!("led_brightness: {}", &focus.led_brightness_get()?);
    println!("led_brightness_underglow: {}", &focus.led_brightness_underglow_get()?);
    println!("led_brightness_wireless: {}", &focus.led_brightness_wireless_get()?);
    println!("led_brightness_underglow_wireless: {}", &focus.led_brightness_underglow_wireless_get()?);
    println!("led_fade: {}", &focus.led_fade_get()?);
    println!("led_theme: {:?}", &focus.led_theme_get()?);
    println!("palette: {:?}", &focus.palette_get()?);
    println!("color_map: {:?}", &focus.color_map_get()?);
    println!("led_idle_true_sleep: {}", &focus.led_idle_true_sleep_get()?);
    println!("led_idle_true_sleep_time: {:?}", &focus.led_idle_true_sleep_time_get()?);
    println!("led_idle_time_limit: {:?}", &focus.led_idle_time_limit_get()?);
    println!("led_idle_wireless: {}", &focus.led_idle_wireless_get()?);
    println!("hardware_version: {}", &focus.hardware_version_get()?);
    println!("qukeys_hold_timeout: {:?}", &focus.qukeys_hold_timeout_get()?);
    println!("qukeys_overlap_threshold: {:?}", &focus.qukeys_overlap_threshold_get()?);
    println!("macros_map: {}", &focus.macros_map_get()?);
    println!("macros_memory: {}", &focus.macros_memory_get()?);
    println!("help: {:#?}", &focus.help_get()?);
    println!("mouse_speed: {}", &focus.mouse_speed_get()?);
    println!("mouse_delay: {:?}", &focus.mouse_delay_get()?);
    println!("mouse_acceleration_speed: {}", &focus.mouse_acceleration_speed_get()?);
    println!("mouse_acceleration_delay: {:?}", &focus.mouse_acceleration_delay_get()?);
    println!("mouse_wheel_speed: {}", &focus.mouse_wheel_speed_get()?);
    println!("mouse_wheel_delay: {:?}", &focus.mouse_wheel_delay_get()?);
    println!("mouse_speed_limit: {}", &focus.mouse_speed_limit_get()?);
    println!("layer_is_active: {}", &focus.layer_is_active_get(0)?);
    println!("layer_state: {:#?}", &focus.layer_state_get()?);
    println!("wireless_battery_level_left: {}", &focus.wireless_battery_level_left_get()?);
    println!("wireless_battery_level_right: {}", &focus.wireless_battery_level_right_get()?);
    println!("wireless_battery_status_left: {}", &focus.wireless_battery_status_left_get()?);
    println!("wireless_battery_status_right: {}", &focus.wireless_battery_status_right_get()?);
    println!("wireless_battery_saving_mode: {}", &focus.wireless_battery_saving_mode_get()?);
    println!("wireless_rf_power_level: {:?}", &focus.wireless_rf_power_level_get()?);
    println!("wireless_rf_channel_hop: {}", &focus.wireless_rf_channel_hop_get()?);
    println!("wireless_rf_sync_pairing: {}", &focus.wireless_rf_sync_pairing_get()?);

    Ok(())
}
```

## Projects using this crate

[Dygma Layer Switcher](https://github.com/mbwilding/dygma-layer-switcher)
