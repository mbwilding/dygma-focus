use crate::errors::*;
use crate::helpers::*;
use crate::prelude::*;
use crate::MAX_LAYERS;
use log::trace;
use std::io::{Read, Write};
use std::str::FromStr;

#[cfg(unix)]
use crate::platform::posix::Focus;
#[cfg(windows)]
use crate::platform::windows::Focus;
#[cfg(target_arch = "wasm32")]
use crate::platform::wasm::Focus;

/// Public methods
impl Focus {
    /// Writes bytes to the serial port.
    pub fn write_bytes(&mut self, bytes: &[u8]) -> Result<(), FocusError> {
        trace!("Writing bytes: {:02X?}", bytes);
        let serial = &mut self.serial;
        serial
            .write_all(bytes)
            .map_err(FocusError::SerialPortWriteError)?;
        serial.flush().map_err(FocusError::SerialPortFlushError)?;
        Ok(())
    }

    /// Response from serial port
    pub fn read_string(&mut self) -> Result<String, FocusError> {
        let eof_marker = b"\r\n.\r\n";
        self.response_buffer.clear();
        let serial = &mut self.serial;
        loop {
            let prev_len = self.response_buffer.len();
            self.response_buffer.resize(prev_len + 1024, 0);
            match serial.read(&mut self.response_buffer[prev_len..]) {
                Ok(0) => continue,
                Ok(size) => {
                    self.response_buffer.truncate(prev_len + size);
                    self.response_buffer.retain(|&x| x != 0);
                    trace!("Received bytes: {:02X?}", &self.response_buffer[..size]);
                    if self.response_buffer.ends_with(eof_marker) {
                        break;
                    }
                }
                Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                Err(e) => return Err(FocusError::SerialPortReadError(e)),
            }
        }
        while let Some(pos) = self
            .response_buffer
            .windows(eof_marker.len())
            .position(|window| window == eof_marker)
        {
            self.response_buffer.drain(pos..pos + eof_marker.len());
        }
        let start = self
            .response_buffer
            .iter()
            .position(|&b| !b.is_ascii_whitespace())
            .unwrap_or(0);
        let end = self
            .response_buffer
            .iter()
            .rposition(|&b| !b.is_ascii_whitespace())
            .map_or(0, |p| p + 1);
        let trimmed_buffer = &self.response_buffer[start..end];
        let response =
            std::str::from_utf8(trimmed_buffer).map_err(FocusError::Utf8ConversionError)?;
        if !response.is_empty() {
            trace!("Command RX: {}", &response);
        } else {
            trace!("Command RX: [Ack]");
        }
        Ok(response.to_string())
    }

    /// Gets the settings from the device.
    pub fn settings_get(&mut self) -> Result<Settings, FocusError> {
        Ok(Settings {
            keymap_custom: self.keymap_custom_get()?,
            keymap_default: self.keymap_default_get()?,
            keymap_only_custom: self.keymap_only_custom_get()?,
            settings_default_layer: self.settings_default_layer_get()?,
            superkeys_map: self.superkeys_map_get()?,
            superkeys_wait_for: self.superkeys_wait_for_get()?,
            superkeys_timeout: self.superkeys_timeout_get()?,
            superkeys_repeat: self.superkeys_repeat_get()?,
            superkeys_hold_start: self.superkeys_hold_start_get()?,
            superkeys_overlap: self.superkeys_overlap_get()?,
            led_mode: self.led_mode_get()?,
            led_brightness_keys_wired: self.led_brightness_top_get()?,
            led_brightness_underglow_wired: self.led_brightness_underglow_wired_get().ok(),
            led_brightness_keys_wireless: self.led_brightness_keys_wireless_get().ok(),
            led_brightness_underglow_wireless: self.led_brightness_underglow_wireless_get().ok(),
            led_fade: self.led_fade_get().ok(),
            led_theme: self.led_theme_get()?,
            palette_rgb: self.palette_rgb_get().ok(),
            palette_rgbw: self.palette_rgbw_get().ok(),
            color_map: self.color_map_get()?,
            led_idle_true_sleep: self.led_idle_true_sleep_get().ok(),
            led_idle_true_sleep_time: self.led_idle_true_sleep_time_get().ok(),
            led_idle_time_limit_wired: self.led_idle_time_limit_wired_get()?,
            led_idle_time_limit_wireless: self.led_idle_time_limit_wireless_get().ok(),
            qukeys_hold_timeout: self.qukeys_hold_timeout_get()?,
            qukeys_overlap_threshold: self.qukeys_overlap_threshold_get()?,
            macros_map: self.macros_map_get()?,
            mouse_speed: self.mouse_speed_get()?,
            mouse_delay: self.mouse_delay_get()?,
            mouse_acceleration_speed: self.mouse_acceleration_speed_get()?,
            mouse_acceleration_delay: self.mouse_acceleration_delay_get()?,
            mouse_wheel_speed: self.mouse_wheel_speed_get()?,
            mouse_wheel_delay: self.mouse_wheel_delay_get()?,
            mouse_speed_limit: self.mouse_speed_limit_get()?,
            wireless_battery_saving_mode: self.wireless_battery_saving_mode_get().ok(),
            wireless_rf_power_level: self.wireless_rf_power_level_get().ok(),
            wireless_rf_channel_hop: self.wireless_rf_channel_hop_get().ok(),
        })
    }

    /// Sets the settings for the device.
    pub fn settings_set(&mut self, settings: &Settings) -> Result<(), FocusError> {
        self.keymap_custom_set(&settings.keymap_custom)?;
        self.keymap_default_set(&settings.keymap_default)?;
        self.keymap_only_custom_set(settings.keymap_only_custom)?;
        self.settings_default_layer_set(settings.settings_default_layer)?;
        self.superkeys_map_set(&settings.superkeys_map)?;
        self.superkeys_wait_for_set(settings.superkeys_wait_for)?;
        self.superkeys_timeout_set(settings.superkeys_timeout)?;
        self.superkeys_repeat_set(settings.superkeys_repeat)?;
        self.superkeys_hold_start_set(settings.superkeys_hold_start)?;
        self.superkeys_overlap_set(settings.superkeys_overlap)?;
        self.led_mode_set(settings.led_mode)?;
        self.led_brightness_top_set(settings.led_brightness_keys_wired)?;
        if let Some(value) = settings.led_brightness_underglow_wired {
            self.led_brightness_underglow_wired_set(value)?;
        }
        if let Some(value) = settings.led_brightness_keys_wireless {
            self.led_brightness_keys_wireless_set(value)?;
        }
        if let Some(value) = settings.led_brightness_underglow_wireless {
            self.led_brightness_underglow_wireless_set(value)?;
        }
        if let Some(value) = settings.led_fade {
            self.led_fade_set(value)?;
        }
        self.led_theme_set(&settings.led_theme)?;
        if let Some(value) = &settings.palette_rgb {
            self.palette_rgb_set(value)?;
        }
        if let Some(value) = &settings.palette_rgbw {
            self.palette_rgbw_set(value)?;
        }
        self.color_map_set(&settings.color_map)?;
        if let Some(value) = settings.led_idle_true_sleep {
            self.led_idle_true_sleep_set(value)?;
        }
        if let Some(value) = settings.led_idle_true_sleep_time {
            self.led_idle_true_sleep_time_set(value)?;
        }
        self.led_idle_time_limit_wired_set(settings.led_idle_time_limit_wired)?;
        if let Some(value) = settings.led_idle_time_limit_wireless {
            self.led_idle_time_limit_wireless_set(value)?;
        }
        self.qukeys_hold_timeout_set(settings.qukeys_hold_timeout)?;
        self.qukeys_overlap_threshold_set(settings.qukeys_overlap_threshold)?;
        self.macros_map_set(&settings.macros_map)?;
        self.mouse_speed_set(settings.mouse_speed)?;
        self.mouse_delay_set(settings.mouse_delay)?;
        self.mouse_acceleration_speed_set(settings.mouse_acceleration_speed)?;
        self.mouse_acceleration_delay_set(settings.mouse_acceleration_delay)?;
        self.mouse_wheel_speed_set(settings.mouse_wheel_speed)?;
        self.mouse_wheel_delay_set(settings.mouse_wheel_delay)?;
        self.mouse_speed_limit_set(settings.mouse_speed_limit)?;
        if let Some(wireless_battery_saving_mode) = settings.wireless_battery_saving_mode {
            self.wireless_battery_saving_mode_set(wireless_battery_saving_mode)?;
        }
        if let Some(wireless_rf_power_level) = settings.wireless_rf_power_level {
            self.wireless_rf_power_level_set(wireless_rf_power_level)?;
        }
        if let Some(wireless_rf_channel_hop) = settings.wireless_rf_channel_hop {
            self.wireless_rf_channel_hop_set(wireless_rf_channel_hop)?;
        }

        Ok(())
    }
}

/// Private methods
impl Focus {
    /// Sends a command to the device.
    fn command_raw(
        &mut self,
        command: &str,
        suffix: Option<char>,
        wait_for_response: bool,
    ) -> Result<(), FocusError> {
        trace!("Command TX: {}", command);

        if let Some(char) = suffix {
            self.write_bytes(format!("{}{}", command, char).as_bytes())?;
        } else {
            self.write_bytes(command.as_bytes())?;
        }

        if wait_for_response {
            let _response = self.read_string()?;
            // It's not necessary to do anything with the response, but we need to wait for it.
        }

        Ok(())
    }

    /// Sends a command to the device, with a single new line ending.
    fn command_new_line(
        &mut self,
        command: &str,
        wait_for_response: bool,
    ) -> Result<(), FocusError> {
        self.command_raw(command, Some('\n'), wait_for_response)
    }

    /// Sends a command to the device, with a single whitespace ending.
    fn command_whitespace(&mut self, command: &str) -> Result<(), FocusError> {
        self.command_raw(command, Some(' '), false)
    }

    /// Sends a command to the device, and returns the response as a string.
    fn command_response_string(&mut self, command: &str) -> Result<String, FocusError> {
        self.command_new_line(command, false)?;
        self.read_string()
    }

    /// Sends a command to the device, and returns the response as a numerical value.
    fn command_response_numerical<T>(&mut self, command: &str) -> Result<T, FocusError>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let response = self.command_response_string(command)?;
        response
            .parse::<T>()
            .map_err(|_| FocusError::ParseNumericalError { string: response })
    }

    /// Sends a command to the device, and returns the response as a boolean value.
    fn command_response_bool(&mut self, command: &str) -> Result<bool, FocusError> {
        let response = self.command_response_string(command)?;
        if response.is_empty() {
            Err(FocusError::EmptyResponseError)
        } else if response == "0" || response == "false" {
            Ok(false)
        } else if response == "1" || response == "true" {
            Ok(true)
        } else {
            return Err(FocusError::ParseBoolError { string: response });
        }
    }

    /// Sends a command to the device, and returns the response as a vector of strings.
    fn command_response_vec_string(&mut self, command: &str) -> Result<Vec<String>, FocusError> {
        Ok(self
            .command_response_string(command)?
            .lines()
            .map(|line| line.replace('\r', ""))
            .collect())
    }
}

/// Public API methods
impl Focus {
    /// Get the version of the firmware.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#version
    pub fn version(&mut self) -> Result<String, FocusError> {
        self.command_response_string("version")
    }

    /// Gets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub fn keymap_custom_get(&mut self) -> Result<Vec<u16>, FocusError> {
        let data = self.command_response_string("keymap.custom")?;
        string_to_numerical_vec(&data)
    }

    /// Sets the whole custom keymap stored in the keyboard.
    ///
    /// Layers 0 and above, The layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub fn keymap_custom_set(&mut self, data: &[u16]) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("keymap.custom {}", numerical_vec_to_string(data)),
            true,
        )
    }

    /// Gets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub fn keymap_default_get(&mut self) -> Result<Vec<u16>, FocusError> {
        let data = self.command_response_string("keymap.default")?;
        string_to_numerical_vec(&data)
    }

    /// Sets the default keymap stored in the keyboard.
    ///
    /// Layers -1 and -2, the layers are -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub fn keymap_default_set(&mut self, data: &[u16]) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("keymap.default {}", numerical_vec_to_string(data)),
            true,
        )
    }

    /// Gets the user setting of hiding the default layers.
    ///
    /// It does not allow you to increment the number of available layers by start using the default ones.
    /// They are there so you can store a backup for two layers in your keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub fn keymap_only_custom_get(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("keymap.onlyCustom")
    }

    /// Sets the user setting of hiding the default layers.
    ///
    /// It does not allow you to increment the number of available layers by start using the default ones.
    /// They are there so you can store a backup for two layers in your keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub fn keymap_only_custom_set(&mut self, state: bool) -> Result<(), FocusError> {
        self.command_new_line(&format!("keymap.onlyCustom {}", state as u8), true)
    }

    /// Gets the default layer the keyboard will boot with.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub fn settings_default_layer_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("settings.defaultLayer")
    }

    /// Sets the default layer the keyboard will boot with.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub fn settings_default_layer_set(&mut self, layer: u8) -> Result<(), FocusError> {
        if layer > MAX_LAYERS {
            return Err(FocusError::ValueAboveLimitError {
                label: "layer",
                max: MAX_LAYERS as usize,
                provided: layer as usize,
            });
        }
        if self.settings_default_layer_get()? == layer {
            return Ok(());
        }
        self.command_new_line(&format!("settings.defaultLayer {}", layer), true)
    }

    /// Gets a boolean value that states true if all checks have been performed on the current settings, and its upload was done in the intended way.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsvalid
    pub fn settings_valid(&mut self) -> Result<bool, FocusError> {
        self.command_response_numerical("settings.valid?")
    }

    /// Gets the current settings version.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub fn settings_version_get(&mut self) -> Result<String, FocusError> {
        self.command_response_string("settings.version")
    }

    /// Sets the current settings version.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub fn settings_version_set(&mut self, version: &str) -> Result<(), FocusError> {
        self.command_new_line(&format!("settings.version {}", version), true)
    }

    /// Gets the CRC checksum of the layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingscrc
    pub fn settings_crc(&mut self) -> Result<String, FocusError> {
        self.command_response_string("settings.crc")
    }

    /// Gets the EEPROM's contents.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub fn eeprom_contents_get(&mut self) -> Result<String, FocusError> {
        self.command_response_string("eeprom.contents")
    }

    /// Sets the EEPROM's contents.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub fn eeprom_contents_set(&mut self, data: &str) -> Result<(), FocusError> {
        self.command_new_line(&format!("eeprom.contents {}", data), true)
    }

    /// Gets the EEPROM's free bytes.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromfree
    pub fn eeprom_free(&mut self) -> Result<String, FocusError> {
        self.command_response_string("eeprom.free")
    }

    pub fn upgrade_start(&mut self) -> Result<(), FocusError> {
        self.command_new_line("upgrade.start", false)
    }

    pub fn upgrade_is_ready(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("upgrade.isReady")
    }

    pub fn upgrade_neuron(&mut self) -> Result<(), FocusError> {
        self.command_new_line("upgrade.neuron", false)
    }

    pub fn upgrade_end(&mut self) -> Result<(), FocusError> {
        self.command_new_line("upgrade.start", false)
    }

    pub fn upgrade_keyscanner_is_connected(&mut self, side: Side) -> Result<bool, FocusError> {
        self.command_response_bool(&format!("upgrade.keyscanner.isConnected {}", side as u8))
    }

    pub fn upgrade_keyscanner_is_bootloader(&mut self, side: Side) -> Result<bool, FocusError> {
        self.command_response_bool(&format!("upgrade.keyscanner.isBootloader {}", side as u8))
    }

    pub fn upgrade_keyscanner_begin(&mut self, side: Side) -> Result<bool, FocusError> {
        self.command_response_bool(&format!("upgrade.keyscanner.begin {}", side as u8))
            .map_err(|_| FocusError::SideDisconnectedError { side })
    }

    pub fn upgrade_keyscanner_is_ready(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("upgrade.keyscanner.isReady")
            .map_err(|_| FocusError::DeviceNotReadyError)
    }

    pub fn upgrade_keyscanner_get_info(&mut self) -> Result<String, FocusError> {
        self.command_response_string("upgrade.keyscanner.getInfo")
    }

    pub fn upgrade_keyscanner_send_write(&mut self) -> Result<(), FocusError> {
        self.command_whitespace("upgrade.keyscanner.sendWrite")
    }

    // TODO: upgrade.keyscanner.validate

    pub fn upgrade_keyscanner_finish(&mut self) -> Result<String, FocusError> {
        self.command_response_string("upgrade.keyscanner.finish")
    }

    // TODO: upgrade.keyscanner.sendStart

    /// Gets the Superkeys map.
    ///
    /// Each action in a Superkey is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc...
    ///
    /// To know more about keycodes and to find the right one for your actions, check the key map database.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub fn superkeys_map_get(&mut self) -> Result<Vec<u16>, FocusError> {
        let data = self.command_response_string("superkeys.map")?;
        string_to_numerical_vec(&data)
    }

    /// Sets the Superkeys map.
    ///
    /// Each action in a Superkey is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc...
    ///
    /// To know more about keycodes and to find the right one for your actions, check the key map database.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub fn superkeys_map_set(&mut self, data: &[u16]) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("superkeys.map {}", numerical_vec_to_string(data)),
            true,
        )
    }

    /// Gets the Superkeys wait for duration in milliseconds.
    ///
    /// Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held,
    ///
    /// So for example,
    /// if the variable is set to 500ms, you can maintain the hold key, it will emit a key code corresponding to the action that it triggers,
    /// then it will wait for wait for time for making another key press with that same key code.
    /// This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub fn superkeys_wait_for_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("superkeys.waitfor")
    }

    /// Sets the Superkeys wait for duration in milliseconds.
    ///
    /// Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held,
    ///
    /// So for example,
    /// if the variable is set to 500ms, you can maintain the hold key, it will emit a key code corresponding to the action that it triggers,
    /// then it will wait for wait for time for making another key press with that same key code.
    /// This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub fn superkeys_wait_for_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("superkeys.waitfor {}", &milliseconds), true)
    }

    /// Gets the Superkeys timeout of how long it waits for the next tap in milliseconds.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub fn superkeys_timeout_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("superkeys.timeout")
    }

    /// Sets the Superkeys timeout of how long it waits for the next tap in milliseconds.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub fn superkeys_timeout_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("superkeys.timeout {}", &milliseconds), true)
    }

    /// Gets the Superkeys repeat duration in milliseconds.
    ///
    /// The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub fn superkeys_repeat_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("superkeys.repeat")
    }

    /// Sets the Superkeys repeat duration in milliseconds.
    ///
    /// The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub fn superkeys_repeat_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("superkeys.repeat {}", &milliseconds), true)
    }

    /// Gets the Superkeys hold start duration in milliseconds.
    ///
    /// The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub fn superkeys_hold_start_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("superkeys.holdstart")
    }

    /// Sets the Superkeys hold start duration in milliseconds.
    ///
    /// The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub fn superkeys_hold_start_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("superkeys.holdstart {}", &milliseconds), true)
    }

    /// Gets the Superkeys overlap percentage.
    ///
    /// The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the superkey.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub fn superkeys_overlap_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("superkeys.overlap")
    }

    /// Sets the Superkeys overlap percentage.
    ///
    /// The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the superkey.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub fn superkeys_overlap_set(&mut self, percentage: u8) -> Result<(), FocusError> {
        if percentage > 80 {
            return Err(FocusError::ValueAboveLimitError {
                label: "percentage",
                max: 80,
                provided: percentage as usize,
            });
        }
        self.command_new_line(&format!("superkeys.overlap {}", percentage), true)
    }

    /// Gets the color of a specific LED.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub fn led_at_get(&mut self, led: u8) -> Result<RGB, FocusError> {
        let response = self.command_response_string(&format!("led.at {}", led))?;
        if response.is_empty() {
            return Err(FocusError::EmptyResponseError);
        }
        let parts = response.split_whitespace().collect::<Vec<&str>>();
        if parts.len() != 3 {
            return Err(FocusError::PartCountError { expected: 3 });
        }
        let r = parts[0].parse().map_err(FocusError::ParseIntError)?;
        let g = parts[1].parse().map_err(FocusError::ParseIntError)?;
        let b = parts[2].parse().map_err(FocusError::ParseIntError)?;
        Ok(RGB { r, g, b })
    }

    /// Sets the color of a specific LED.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub fn led_at_set(&mut self, led: u8, color: &RGB) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("led.at {} {} {} {}", led, color.r, color.g, color.b),
            true,
        )
    }

    /// Sets the color of all the LEDs.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledsetall
    pub fn led_all(&mut self, color: &RGB) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("led.setAll {} {} {}", color.r, color.g, color.b,),
            true,
        )
    }

    /// Gets the LED mode.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub fn led_mode_get(&mut self) -> Result<LedMode, FocusError> {
        self.command_response_numerical("led.mode")
    }

    /// Sets the LED mode.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub fn led_mode_set(&mut self, mode: LedMode) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.mode {}", mode as u8), true)
    }

    /// Gets the key LED brightness (wired).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_top_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("led.brightness")
    }

    /// Sets the key LED brightness (wired).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_top_set(&mut self, brightness: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.brightness {}", brightness), true)
    }

    /// Gets the underglow LED brightness (wired).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_wired_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("led.brightnessUG")
    }

    /// Sets the underglow LED brightness (wired).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_wired_set(&mut self, brightness: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.brightnessUG {}", brightness), true)
    }

    /// Gets the key LED brightness (wireless).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnesswireless
    pub fn led_brightness_keys_wireless_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("led.brightness.wireless")
    }

    /// Sets the key LED brightness (wireless).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnesswireless
    pub fn led_brightness_keys_wireless_set(&mut self, brightness: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.brightness.wireless {}", brightness), true)
    }

    /// Gets the underglow LED brightness (wireless).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessugwireless
    pub fn led_brightness_underglow_wireless_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("led.brightnessUG.wireless")
    }

    /// Sets the underglow LED brightness (wireless).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessugwireless
    pub fn led_brightness_underglow_wireless_set(
        &mut self,
        brightness: u8,
    ) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.brightnessUG.wireless {}", brightness), true)
    }

    /// Gets the LED fade.
    pub fn led_fade_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("led.fade")
    }

    /// Sets the LED fade.
    pub fn led_fade_set(&mut self, fade: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.fade {}", fade), true)
    }

    /// Gets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub fn led_theme_get(&mut self) -> Result<Vec<RGB>, FocusError> {
        let data = self.command_response_string("led.theme")?;
        string_to_rgb_vec(&data)
    }

    /// Sets the LED theme.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub fn led_theme_set(&mut self, data: &[RGB]) -> Result<(), FocusError> {
        self.command_new_line(&format!("led.theme {}", &rgb_vec_to_string(data)), true)
    }

    /// Gets the palette as RGB.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_rgb_get(&mut self) -> Result<Vec<RGB>, FocusError> {
        let data = self.command_response_string("palette")?;
        string_to_rgb_vec(&data)
    }

    /// Sets the palette as RGB.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_rgb_set(&mut self, data: &[RGB]) -> Result<(), FocusError> {
        self.command_new_line(&format!("palette {}", rgb_vec_to_string(data)), true)
    }

    /// Gets the palette as RGBW.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_rgbw_get(&mut self) -> Result<Vec<RGBW>, FocusError> {
        let data = self.command_response_string("palette")?;
        string_to_rgbw_vec(&data)
    }

    /// Sets the palette as RGBW.
    ///
    /// The color palette is used by the color map to establish each color that can be assigned to the keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_rgbw_set(&mut self, data: &[RGBW]) -> Result<(), FocusError> {
        self.command_new_line(&format!("palette {}", rgbw_vec_to_string(data)), true)
    }

    /// Gets the color map.
    ///
    /// This command reads the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub fn color_map_get(&mut self) -> Result<Vec<u8>, FocusError> {
        let data = self.command_response_string("colormap.map")?;
        string_to_numerical_vec(&data)
    }

    /// Sets the color map.
    ///
    /// This command writes the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub fn color_map_set(&mut self, data: &[u8]) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("colormap.map {}", numerical_vec_to_string(data)),
            true,
        )
    }

    /// Gets the idle LED true sleep state.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstrue_sleep
    pub fn led_idle_true_sleep_get(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("idleleds.true_sleep")
    }

    /// Sets the idle LED true sleep state.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstrue_sleep
    pub fn led_idle_true_sleep_set(&mut self, state: bool) -> Result<(), FocusError> {
        self.command_new_line(&format!("idleleds.true_sleep {}", state as u8), true)
    }

    /// Gets the idle LED true sleep time in seconds.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstrue_sleep_time
    pub fn led_idle_true_sleep_time_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("idleleds.true_sleep_time")
    }

    /// Sets the idle LED true sleep time in seconds.
    ///
    /// Max: 65,000
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstrue_sleep_time
    pub fn led_idle_true_sleep_time_set(&mut self, seconds: u16) -> Result<(), FocusError> {
        if seconds > 65_000 {
            return Err(FocusError::ValueAboveLimitError {
                label: "seconds",
                max: 65_000,
                provided: seconds as usize,
            });
        }
        self.command_new_line(&format!("idleleds.true_sleep_time {}", seconds), true)
    }

    /// Gets the idle LED wired time limit in seconds.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub fn led_idle_time_limit_wired_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("idleleds.time_limit")
    }

    /// Sets the idle LED wired time limit in seconds.
    ///
    /// Max: 65,000
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub fn led_idle_time_limit_wired_set(&mut self, seconds: u16) -> Result<(), FocusError> {
        if seconds > 65_000 {
            return Err(FocusError::ValueAboveLimitError {
                label: "seconds",
                max: 65_000,
                provided: seconds as usize,
            });
        }
        self.command_new_line(&format!("idleleds.time_limit {}", seconds), true)
    }

    /// Gets the idle LED time limit in seconds (wireless).
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledswireless
    pub fn led_idle_time_limit_wireless_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("idleleds.wireless")
    }

    /// Sets the idle LED time limit in seconds (wireless).
    ///
    /// Max: 65,000
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledswireless
    pub fn led_idle_time_limit_wireless_set(&mut self, seconds: u16) -> Result<(), FocusError> {
        if seconds > 65_000 {
            return Err(FocusError::ValueAboveLimitError {
                label: "seconds",
                max: 65_000,
                provided: seconds as usize,
            });
        }
        self.command_new_line(&format!("idleleds.wireless {}", seconds), true)
    }

    /// Gets the keyboard model name.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareversion
    pub fn hardware_version_get(&mut self) -> Result<String, FocusError> {
        self.command_response_string("hardware.version")
    }

    /// Sets the keyboard model name.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareversion
    pub fn hardware_version_set(&mut self, data: &str) -> Result<(), FocusError> {
        self.command_new_line(&format!("hardware.version {}", data), true)
    }

    // TODO: hardware.side_power https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_power
    // TODO: hardware.side_ver https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_ver
    // TODO: hardware.keyscanInterval
    // TODO: hardware.firmware https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwarefirmware
    // TODO: hardware.chip_id
    // TODO: hardware.chip_info

    /// Gets the Qukeys hold timeout in milliseconds.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub fn qukeys_hold_timeout_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("qukeys.holdTimeout")
    }

    /// Sets the Qukeys hold timeout in milliseconds.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub fn qukeys_hold_timeout_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("qukeys.holdTimeout {}", &milliseconds), true)
    }

    /// Gets the Qukeys overlap threshold in milliseconds.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub fn qukeys_overlap_threshold_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("qukeys.overlapThreshold")
    }

    /// Sets the Qukeys overlap threshold in milliseconds.
    ///
    /// https://kaleidoscope.readthedocs.io/en/latest/plugins/Kaleidoscope-Qukeys.html
    pub fn qukeys_overlap_threshold_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("qukeys.overlapThreshold {}", &milliseconds), true)
    }

    /// Gets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub fn macros_map_get(&mut self) -> Result<Vec<u8>, FocusError> {
        let data = self.command_response_string("macros.map")?;
        string_to_numerical_vec(&data)
    }

    /// Sets the macros map.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub fn macros_map_set(&mut self, data: &[u8]) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("macros.map {}", numerical_vec_to_string(data)),
            true,
        )
    }

    /// Triggers a macro.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrostrigger
    pub fn macros_trigger(&mut self, macro_id: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("macros.trigger {}", macro_id), true)
    }

    /// Gets the macros memory size in bytes.
    pub fn macros_memory(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("macros.memory")
    }

    /// Gets all the available commands in the current version of the serial protocol.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#help
    pub fn help(&mut self) -> Result<Vec<String>, FocusError> {
        self.command_response_vec_string("help")
    }

    /// Gets the virtual mouse speed.
    pub fn mouse_speed_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("mouse.speed")
    }

    /// Sets the virtual mouse speed.
    ///
    /// Max: 127
    pub fn mouse_speed_set(&mut self, speed: u8) -> Result<(), FocusError> {
        if speed > 127 {
            return Err(FocusError::ValueAboveLimitError {
                label: "speed",
                max: 127,
                provided: speed as usize,
            });
        }
        self.command_new_line(&format!("mouse.speed {}", speed), true)
    }

    /// Gets the virtual mouse delay in milliseconds.
    pub fn mouse_delay_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("mouse.speedDelay")
    }

    /// Sets the virtual mouse delay in milliseconds.
    pub fn mouse_delay_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.speedDelay {}", &milliseconds), true)
    }

    /// Gets the virtual mouse acceleration speed.
    pub fn mouse_acceleration_speed_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("mouse.accelSpeed")
    }

    /// Sets the virtual mouse acceleration speed.
    pub fn mouse_acceleration_speed_set(&mut self, speed: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.accelSpeed {}", speed), true)
    }

    /// Gets the virtual mouse acceleration delay in milliseconds.
    pub fn mouse_acceleration_delay_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("mouse.accelDelay")
    }

    /// Sets the virtual mouse acceleration delay in milliseconds.
    pub fn mouse_acceleration_delay_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.accelDelay {}", &milliseconds), true)
    }

    /// Gets the virtual mouse wheel speed.
    pub fn mouse_wheel_speed_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("mouse.wheelSpeed")
    }

    /// Sets the virtual mouse wheel speed.
    pub fn mouse_wheel_speed_set(&mut self, speed: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.wheelSpeed {}", speed), true)
    }

    /// Gets the virtual mouse wheel delay in milliseconds.
    pub fn mouse_wheel_delay_get(&mut self) -> Result<u16, FocusError> {
        self.command_response_numerical("mouse.wheelDelay")
    }

    /// Sets the virtual mouse wheel delay in milliseconds.
    pub fn mouse_wheel_delay_set(&mut self, milliseconds: u16) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.wheelDelay {}", &milliseconds), true)
    }

    /// Gets the virtual mouse speed limit.
    pub fn mouse_speed_limit_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("mouse.speedLimit")
    }

    /// Sets the virtual mouse speed limit.
    pub fn mouse_speed_limit_set(&mut self, limit: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("mouse.speedLimit {}", limit), true)
    }

    /// Activate a certain layer remotely just by sending its order number.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// This does not affect the memory usage as the value is stored in RAM.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layeractivate
    pub fn layer_activate(&mut self, layer: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("layer.activate {}", layer), true)
    }

    /// Deactivate the last layer that the keyboard switched to.
    /// This same function is the way the shift to layer key works on the keyboard.
    ///
    /// Just provide the layer number to make the keyboard go back one layer. The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerdeactivate
    pub fn layer_deactivate(&mut self, layer: Option<u8>) -> Result<(), FocusError> {
        if let Some(layer) = layer {
            if layer > MAX_LAYERS {
                return Err(FocusError::ValueAboveLimitError {
                    label: "layer",
                    max: MAX_LAYERS as usize,
                    provided: layer as usize,
                });
            }
            self.command_new_line(&format!("layer.deactivate {}", layer), true)?
        }
        self.command_new_line("layer.deactivate", true)
    }

    /// Gets the state of the provided layer.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerisactive
    pub fn layer_is_active(&mut self, layer: u8) -> Result<bool, FocusError> {
        if layer > MAX_LAYERS {
            return Err(FocusError::ValueAboveLimitError {
                label: "layer",
                max: MAX_LAYERS as usize,
                provided: layer as usize,
            });
        }
        self.command_response_bool(&format!("layer.isActive {}", layer))
    }

    /// Switch to a certain layer.
    ///
    /// The layer is -1 to Bazecor.
    ///
    /// The difference between this command and the layer_activate alternative, is that the layer_activate adds to the layer switching history, but moveTo will erase that memory and return it to an array length 1 and holding the current layer the keyboard moved to.
    ///
    /// This does not affect the memory usage as the value is stored in RAM.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layermoveto
    pub fn layer_move_to(&mut self, layer: u8) -> Result<(), FocusError> {
        self.command_new_line(&format!("layer.moveTo {}", layer), true)
    }

    /// Gets the status for up to 32 layers.
    ///
    /// It will return a vector of bools with the respective index matching each layer, -1 from Bazecor.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerstate
    pub fn layer_state(&mut self) -> Result<Vec<bool>, FocusError> {
        let response = self.command_response_string("layer.state")?;
        let parts = response.split_whitespace().collect::<Vec<&str>>();
        let nums = parts.iter().map(|&part| part == "1").collect();
        Ok(nums)
    }

    /// Gets the battery level of the left keyboard as a percentage.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatteryleftlevel
    pub fn wireless_battery_level_left_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("wireless.battery.left.level")
    }

    /// Gets the battery level of the right keyboard as a percentage.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatteryrightlevel
    pub fn wireless_battery_level_right_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("wireless.battery.right.level")
    }

    /// Gets the battery status of the left keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatteryleftstatus
    pub fn wireless_battery_status_left_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("wireless.battery.left.status")
    }

    /// Gets the battery status of the right keyboard.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatteryrightstatus
    pub fn wireless_battery_status_right_get(&mut self) -> Result<u8, FocusError> {
        self.command_response_numerical("wireless.battery.right.status")
    }

    /// Gets the battery saving mode state.
    ///
    /// This will be automatically enabled when remaining battery charge is low, but it can be manually enabled earlier.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatterysavingmode
    pub fn wireless_battery_saving_mode_get(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("wireless.battery.savingMode")
    }

    /// Sets the battery saving mode state.
    ///
    /// This will be automatically enabled when remaining battery charge is low, but it can be manually enabled earlier.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatterysavingmode
    pub fn wireless_battery_saving_mode_set(&mut self, state: bool) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("wireless.battery.savingMode {}", state as u8),
            true,
        )
    }

    /// Forces the neuron to update the battery level.
    ///
    /// This typically takes a second or two to update the values for the wireless_battery_level commands to read.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessbatteryforceread
    pub fn wireless_battery_force_read(&mut self) -> Result<(), FocusError> {
        self.command_new_line("wireless.battery.forceRead", false)
    }

    /// Gets the RF power level.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessrfpower
    pub fn wireless_rf_power_level_get(&mut self) -> Result<WirelessPowerMode, FocusError> {
        self.command_response_numerical("wireless.rf.power")
    }

    /// Sets the RF power level.
    pub fn wireless_rf_power_level_set(
        &mut self,
        wireless_power_mode: WirelessPowerMode,
    ) -> Result<(), FocusError> {
        self.command_new_line(
            &format!("wireless.rf.power {}", wireless_power_mode as u8),
            true,
        )
    }

    /// Gets the RF channel hop state.
    pub fn wireless_rf_channel_hop_get(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("wireless.rf.channelHop")
    }

    /// Sets the RF channel hop state.
    pub fn wireless_rf_channel_hop_set(&mut self, state: bool) -> Result<(), FocusError> {
        self.command_new_line(&format!("wireless.rf.channelHop {}", state as u8), true)
    }

    /// Gets the sync pairing state.
    ///
    /// https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#wirelessrfsyncpairing
    pub fn wireless_rf_sync_pairing(&mut self) -> Result<bool, FocusError> {
        self.command_response_bool("wireless.rf.syncPairing")
    }
}
