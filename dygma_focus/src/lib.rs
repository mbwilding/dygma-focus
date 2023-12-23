extern crate core;

pub mod devices;
pub mod enums;
pub mod prelude;
pub mod structs;

use crate::prelude::*;
use anyhow::{anyhow, bail, Result};
use devices::DEVICES;
use serialport::{SerialPort, SerialPortType};
use std::str::FromStr;
use std::time::Duration;
use tracing::{debug, error, trace};

pub struct Focus {
    port: Option<Box<dyn SerialPort>>,
    response_buffer: Vec<u8>,
}

impl Focus {
    /// Create a new instance of the Focus API.
    pub fn new() -> Self {
        Self {
            port: None,
            response_buffer: Vec::with_capacity(4096),
        }
    }
}

impl Default for Focus {
    /// Create a new instance of the Focus API.
    fn default() -> Self {
        Self::new()
    }
}

impl Focus {
    /// Find all supported devices.
    pub fn device_find_all(&self) -> Result<Vec<Device>> {
        let ports = match serialport::available_ports() {
            Ok(ports) => ports,
            Err(e) => {
                let err_msg = format!("Failed to enumerate serial ports: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        debug!("Available serial ports: {:?}", ports);

        let found_devices: Vec<Device> = ports
            .into_iter()
            .filter_map(|port| match &port.port_type {
                SerialPortType::UsbPort(info) => DEVICES
                    .iter()
                    .find(|&device| device.vendor_id == info.vid && device.product_id == info.pid)
                    .map(|device| Device {
                        name: device.name,
                        port: port.port_name,
                    }),
                _ => None,
            })
            .collect();

        debug!("Found devices: {:?}", found_devices);

        Ok(found_devices)
    }

    /// Find the first supported device.
    pub fn device_find_first(&self) -> Result<Device> {
        let devices = match self.device_find_all() {
            Ok(devices) => devices,
            Err(e) => {
                let err_msg = format!("No device found: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        let device = devices.first().ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        Ok(device.clone())
    }

    /// Open a connection to the keyboard via the first supported device found.
    pub fn device_open_first(&mut self) -> Result<()> {
        self.device_open_via_device(self.device_find_all()?.first().ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
    }

    /// Open a connection to the keyboard via port.
    pub fn device_open_via_port(&mut self, port: &str) -> Result<()> {
        let port_settings = serialport::new(port, 115_200)
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_millis(50));

        let mut port = port_settings.open().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        port.write_data_terminal_ready(true)?;

        self.port = Some(port);

        Ok(())
    }

    /// Open a connection to the keyboard via device.
    pub fn device_open_via_device(&mut self, device: &Device) -> Result<()> {
        self.device_open_via_port(&device.port)
    }

    /// Sends a command to the device, with no response.
    fn command(&mut self, command: &str) -> Result<()> {
        trace!("Command TX: {}", command);

        if let Some(ref mut port) = self.port {
            port.write_all(format!("{}\n", command).as_bytes())?;

            Ok(())
        } else {
            bail!("Serial port is not open");
        }
    }

    /// Sends a command to the device, and returns the response as a string.
    fn command_response_string(&mut self, command: &str) -> Result<String> {
        self.command(command)?;

        let eof_marker = b"\r\n.\r\n";

        if let Some(ref mut port) = self.port {
            self.response_buffer.clear();

            loop {
                let prev_len = self.response_buffer.len();
                self.response_buffer.resize(prev_len + 1024, 0);
                match port.read(&mut self.response_buffer[prev_len..]) {
                    Ok(0) => continue,
                    Ok(size) => {
                        self.response_buffer.truncate(prev_len + size);

                        if self.response_buffer.ends_with(eof_marker) {
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => bail!("Error reading from serial port: {:?}", e),
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

            let response = String::from_utf8(trimmed_buffer.to_vec())
                .map_err(|e| anyhow!("Failed to convert response to UTF-8 string: {:?}", e))?;

            trace!("Command RX: {}", response);

            Ok(response)
        } else {
            Err(anyhow!("Serial port is not open"))
        }
    }

    /// Sends a command to the device, and returns the response as a numerical value.
    fn command_response_numerical<T>(&mut self, command: &str) -> Result<T>
    where
        T: FromStr,
        <T as FromStr>::Err: std::fmt::Debug,
    {
        let response = self.command_response_string(command)?;
        response
            .parse::<T>()
            .map_err(|e| anyhow!("Failed to parse response: {:?}", e))
    }

    /// Sends a command to the device, and returns the response as a boolean value.
    fn command_response_bool(&mut self, command: &str) -> Result<bool> {
        let response = self.command_response_string(command)?;
        Ok(response == "1")
    }

    /// Sends a command to the device, and returns the response as a vector of strings.
    fn command_response_vec_string(&mut self, command: &str) -> Result<Vec<String>> {
        Ok(self
            .command_response_string(command)?
            .lines()
            .map(|line| line.replace('\r', ""))
            .collect())
    }

    /// Get the version of the firmware. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#version
    pub fn version_get(&mut self) -> Result<String> {
        self.command_response_string("version")
    }

    /// Gets the whole custom keymap stored in the keyboard. (Layers 0-9). https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub fn keymap_custom_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.custom")
    }

    /// Sets the whole custom keymap stored in the keyboard. (Layers 0-9). https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapcustom
    pub fn keymap_custom_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("keymap.custom {}", data))
    }

    /// Gets the default keymap stored in the keyboard. (Layers -1 and -2). https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub fn keymap_default_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.default")
    }

    /// Sets the default keymap stored in the keyboard. (Layers -1 and -2). https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymapdefault
    pub fn keymap_default_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("keymap.default {}", data))
    }

    /// Returns true or false depending on the user setting of hiding the default layers or not, it does not allow you to increment the number of available layers by start using the default ones, they are there so you can store a backup for two layers in your keyboard. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub fn keymap_only_custom_get(&mut self) -> Result<bool> {
        self.command_response_bool("keymap.onlyCustom")
    }

    /// Sets the user setting of hiding the default layers or not, it does not allow you to increment the number of available layers by start using the default ones, they are there so you can store a backup for two layers in your keyboard. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#keymaponlycustom
    pub fn keymap_only_custom_set(&mut self, state: bool) -> Result<()> {
        self.command(&format!("keymap.onlyCustom {}", state as u8))
    }

    /// Returns the default layer the keyboard will boot with. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub fn settings_default_layer_get(&mut self) -> Result<i8> {
        self.command_response_numerical("settings.defaultLayer")
    }

    /// Sets the default layer the keyboard will boot with. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsdefaultlayer
    pub fn settings_default_layer_set(&mut self, layer: i8) -> Result<()> {
        self.command(&format!("settings.defaultLayer {}", layer))
    }

    /// Returns a boolean value that states true if all checks have been performed on the current settings and its upload was done in the intended way. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsvalid
    pub fn settings_valid_get(&mut self) -> Result<bool> {
        self.command_response_numerical("settings.valid?")
    }

    /// Gets the current settings version. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub fn settings_version_get(&mut self) -> Result<String> {
        self.command_response_string("settings.version")
    }

    /// Sets the current settings version. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingsversion
    pub fn settings_version_set(&mut self, version: &str) -> Result<()> {
        self.command(&format!("settings.version {}", version))
    }

    /// Gets the current settings version. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#settingscrc
    pub fn settings_crc_get(&mut self) -> Result<String> {
        self.command_response_string("settings.crc")
    }

    /// Gets the EEPROM's contents. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub fn eeprom_contents_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.contents")
    }

    /// Sets the EEPROM's contents. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromcontents
    pub fn eeprom_contents_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("eeprom.contents {}", data))
    }

    /// Gets the EEPROM's free bytes. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#eepromfree
    pub fn eeprom_free_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.free")
    }

    // TODO upgrade.start
    // TODO upgrade.neuron
    // TODO upgrade.end
    // TODO upgrade.keyscanner.isConnected
    // TODO upgrade.keyscanner.isBootloader
    // TODO upgrade.keyscanner.begin
    // TODO upgrade.keyscanner.isReady
    // TODO upgrade.keyscanner.getInfo
    // TODO upgrade.keyscanner.sendWrite
    // TODO upgrade.keyscanner.validate
    // TODO upgrade.keyscanner.finish
    // TODO upgrade.keyscanner.sendStart

    /// Gets the super keys map. This command reads the super keys map, each action in a super key is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc... to know more about keycodes and to find the right one for your actions, check the key map database. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub fn super_keys_map_get(&mut self) -> Result<String> {
        self.command_response_string("superkeys.map")
    }

    /// Sets the super keys map. This command writes the super keys map (1024 bytes of max length), each action in a super key is represented by a key code number that encodes the action, for example if you use the number 44, you are encoding space, etc... to know more about keycodes and to find the right one for your actions, check the key map database. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysmap
    pub fn super_keys_map_set(&mut self, data: &str) -> Result<()> {
        if data.len() > 1_024 {
            bail!("Data must be 1024 bytes or less: {}", data.len());
        }
        self.command(&format!("superkeys.map {}", data))
    }

    /// Gets the wait for value in milliseconds of the keyboard to alter the behaviour of the super keys. Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held, so for example, if the variable is set to 500ms, you can maintain the hold key, it will emmit a key code corresponding to the action that it triggers, then it will wait for wait for time for making another key press with that same key code. This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub fn super_keys_wait_for_get(&mut self) -> Result<u16> {
        self.command_response_numerical("superkeys.waitfor")
    }

    /// Sets the wait for value in milliseconds of the keyboard to alter the behaviour of the super keys. Wait for value specifies the time between the first and subsequent releases of the HOLD actions meanwhile is held, so for example, if the variable is set to 500ms, you can maintain the hold key, it will emmit a key code corresponding to the action that it triggers, then it will wait for wait for time for making another key press with that same key code. This enables the user to delay the hold "machinegun" to be able to release the key and achieve a single keypress from a hold action. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeyswaitfor
    pub fn super_keys_wait_for_set(&mut self, milliseconds: &str) -> Result<()> {
        self.command(&format!("superkeys.waitfor {}", milliseconds))
    }

    /// Gets the timeout of how long super keys waits for the next tap. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub fn super_keys_timeout_get(&mut self) -> Result<u16> {
        self.command_response_numerical("superkeys.timeout")
    }

    /// Sets the timeout of how long super keys waits for the next tap. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeystimeout
    pub fn super_keys_timeout_set(&mut self, milliseconds: u16) -> Result<()> {
        self.command(&format!("superkeys.timeout {}", milliseconds))
    }

    /// Gets the repeat value of the keyboard to alter the behaviour of the super keys. The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub fn super_keys_repeat_get(&mut self) -> Result<u16> {
        self.command_response_numerical("superkeys.repeat")
    }

    /// Sets the repeat value of the keyboard to alter the behaviour of the super keys. The repeat value specifies the time between the second and subsequent key code releases when on hold, it only takes effect after the wait for timer has been exceeded. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysrepeat
    pub fn super_keys_repeat_set(&mut self, milliseconds: u16) -> Result<()> {
        self.command(&format!("superkeys.repeat {}", milliseconds))
    }

    /// Gets the hold start value of the keyboard to alter the behaviour of the super keys. The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub fn super_keys_hold_start_get(&mut self) -> Result<u16> {
        self.command_response_numerical("superkeys.holdstart")
    }

    /// Sets the hold start value of the keyboard to alter the behaviour of the super keys. The hold start value specifies the minimum time that has to pass between the first key down and any other action to trigger a hold, if held it will emit a hold action. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysholdstart
    pub fn super_keys_hold_start_set(&mut self, milliseconds: u16) -> Result<()> {
        self.command(&format!("superkeys.holdstart {}", milliseconds))
    }

    /// Gets the overlap percentage of the keyboard to alter the behaviour of the super keys. The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the super key. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub fn super_keys_overlap_get(&mut self) -> Result<u8> {
        self.command_response_numerical("superkeys.overlap")
    }

    /// Sets the overlap percentage of the keyboard to alter the behaviour of the super keys. The overlap value specifies the percentage of overlap when fast typing that is allowed to happen before triggering a hold action to the overlapped key pressed after the super key. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#superkeysoverlap
    pub fn super_keys_overlap_set(&mut self, percentage: u8) -> Result<()> {
        if percentage > 80 {
            bail!("Percentage must be 80 or below: {}", percentage);
        }
        self.command(&format!("superkeys.overlap {}", percentage))
    }

    /// Gets the color of a specific LED. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub fn led_at_get(&mut self, led: u16) -> Result<Color> {
        let response = self.command_response_string(&format!("led.at {}", led))?;

        if response.is_empty() {
            bail!("Empty response");
        }

        let parts = response.split_whitespace().collect::<Vec<&str>>();

        if parts.len() != 3 {
            bail!("Response does not contain exactly three parts");
        }

        let r = parts[0].parse()?;
        let g = parts[1].parse()?;
        let b = parts[2].parse()?;

        Ok(Color { r, g, b })
    }

    /// Sets the color of a specific LED. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledat
    pub fn led_at_set(&mut self, led: u8, color: Color) -> Result<()> {
        self.command(&format!(
            "led.at {} {} {} {}",
            led, color.r, color.g, color.b
        ))
    }

    /// Sets the color of all the LEDs. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledsetall
    pub fn led_all_set(&mut self, color: Color) -> Result<()> {
        self.command(&format!("led.setAll {} {} {}", color.r, color.g, color.b,))
    }

    /// Gets the LED mode. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub fn led_mode_get(&mut self) -> Result<LedMode> {
        self.command_response_numerical("led.mode")
    }

    /// Sets the LED mode. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledmode
    pub fn led_mode_set(&mut self, mode: LedMode) -> Result<()> {
        self.command(&format!("led.mode {}", mode.value()))
    }

    /// Gets the LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness")
    }

    /// Sets the LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_set(&mut self, brightness: u8) -> Result<()> {
        self.command(&format!("led.brightness {}", brightness))
    }

    /// Gets the underglow LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightnessUG")
    }

    /// Sets the underglow LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_set(&mut self, brightness: u8) -> Result<()> {
        self.command(&format!("led.brightnessUG {}", brightness))
    }

    /// Gets the wireless LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_wireless_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightness.wireless")
    }

    /// Sets the wireless LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightness
    pub fn led_brightness_wireless_set(&mut self, brightness: u8) -> Result<()> {
        self.command(&format!("led.brightness.wireless {}", brightness))
    }

    /// Gets the wireless underglow LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_wireless_get(&mut self) -> Result<u8> {
        self.command_response_numerical("led.brightnessUG.wireless")
    }

    /// Sets the wireless underglow LED brightness. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledbrightnessug
    pub fn led_brightness_underglow_wireless_set(&mut self, brightness: u8) -> Result<()> {
        self.command(&format!("led.brightnessUG.wireless {}", brightness))
    }

    /// Gets the LED fade. Undocumented.
    pub fn led_fade_get(&mut self) -> Result<u16> {
        self.command_response_numerical("led.fade")
    }

    /// Sets the LED fade. Undocumented.
    pub fn led_fade_set(&mut self, fade: u16) -> Result<()> {
        self.command(&format!("led.fade {}", fade))
    }

    /// Gets the LED theme. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub fn led_theme_get(&mut self) -> Result<String> {
        self.command_response_string("led.theme")
    }

    /// Sets the LED theme. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#ledtheme
    pub fn led_theme_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("led.theme {}", data))
    }

    /// Gets the palette. The color palette is used by the color map to establish each color that can be assigned to the keyboard. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_get(&mut self) -> Result<String> {
        self.command_response_string("palette")
    }

    /// Sets the palette. The color palette is used by the color map to establish each color that can be assigned to the keyboard. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#palette
    pub fn palette_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("palette {}", data))
    }

    /// Gets the color map. This command reads the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub fn color_map_get(&mut self) -> Result<String> {
        self.command_response_string("colormap.map")
    }

    /// Sets the color map. This command writes the color map that assigns each color listed in the palette to individual LEDs, mapping them to the keyboard's current layout. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#colormapmap
    pub fn color_map_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("colormap.map {}", data))
    }

    /// Gets the idle LED true sleep state.
    pub fn led_idle_true_sleep_get(&mut self) -> Result<bool> {
        self.command_response_bool("idleleds.true_sleep")
    }

    /// Sets the idle LED true sleep state.
    pub fn led_idle_true_sleep_set(&mut self, state: bool) -> Result<()> {
        self.command(&format!("idleleds.true_sleep {}", state as u8))
    }

    /// Gets the idle LED true sleep time in seconds.
    pub fn led_idle_true_sleep_time_get(&mut self) -> Result<u16> {
        self.command_response_numerical("idleleds.true_sleep_time")
    }

    /// Sets the idle LED true sleep time in seconds.
    pub fn led_idle_true_sleep_time_set(&mut self, seconds: u16) -> Result<()> {
        if seconds > 65_000 {
            bail!("Seconds must be 65000 or below: {}", seconds);
        }
        self.command(&format!("idleleds.true_sleep_time {}", seconds))
    }

    /// Gets the idle LED time limit in seconds. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub fn led_idle_time_limit_get(&mut self) -> Result<u16> {
        self.command_response_numerical("idleleds.time_limit")
    }

    /// Sets the idle LED time limit in seconds. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#idleledstime_limit
    pub fn led_idle_time_limit_set(&mut self, seconds: u16) -> Result<()> {
        if seconds > 65_000 {
            bail!("Seconds must be 65000 or below: {}", seconds);
        }
        self.command(&format!("idleleds.time_limit {}", seconds))
    }

    // TODO idleleds.wireless

    /// Gets the keyboard model name. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareversion
    pub fn hardware_version_get(&mut self) -> Result<String> {
        self.command_response_string("hardware.version")
    }

    // TODO hardware.side_power https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_power
    // TODO hardware.side_ver https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwareside_ver
    // TODO hardware.keyscanInterval
    // TODO hardware.firmware https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#hardwarefirmware
    // TODO hardware.chip_id
    // TODO hardware.chip_info
    // TODO qukeys.holdTimeout
    // TODO qukeys.overlapThreshold

    /// Gets the macros map. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub fn macros_map_get(&mut self) -> Result<String> {
        self.command_response_string("macros.map")
    }

    /// Sets the macros map. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrosmap
    pub fn macros_map_set(&mut self, data: &str) -> Result<()> {
        if data.len() > 2_048 {
            bail!("Data must be 1024 bytes or less: {}", data.len());
        }
        self.command(&format!("macros.map {}", data))
    }

    /// Triggers a macro. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#macrostrigger
    pub fn macros_trigger(&mut self, macro_id: u8) -> Result<()> {
        self.command(&format!("macros.trigger {}", macro_id))
    }

    /// Gets the macros memory size in bytes.
    pub fn macros_memory_get(&mut self) -> Result<u16> {
        self.command_response_numerical("macros.memory")
    }

    /// Gets all the available commands in the current version of the serial protocol. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#help
    pub fn help_get(&mut self) -> Result<Vec<String>> {
        self.command_response_vec_string("help")
    }

    /// Gets the virtual mouse speed.
    pub fn mouse_speed_get(&mut self) -> Result<i8> {
        self.command_response_numerical("mouse.speed")
    }

    /// Sets the virtual mouse speed.
    pub fn mouse_speed_set(&mut self, speed: i8) -> Result<()> {
        self.command(&format!("mouse.speed {}", speed))
    }

    /// Gets the virtual mouse delay.
    pub fn mouse_delay_get(&mut self) -> Result<i8> {
        self.command_response_numerical("mouse.speedDelay")
    }

    /// Sets the virtual mouse delay.
    pub fn mouse_delay_set(&mut self, delay: i8) -> Result<()> {
        self.command(&format!("mouse.speedDelay {}", delay))
    }

    /// Gets the virtual mouse acceleration speed.
    pub fn mouse_acceleration_speed_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.accelSpeed")
    }

    /// Sets the virtual mouse acceleration speed.
    pub fn mouse_acceleration_speed_set(&mut self, speed: u8) -> Result<()> {
        self.command(&format!("mouse.accelSpeed {}", speed))
    }

    /// Gets the virtual mouse acceleration delay.
    pub fn mouse_acceleration_delay_get(&mut self) -> Result<i8> {
        self.command_response_numerical("mouse.accelDelay")
    }

    /// Sets the virtual mouse acceleration delay.
    pub fn mouse_acceleration_delay_set(&mut self, delay: i8) -> Result<()> {
        self.command(&format!("mouse.accelDelay {}", delay))
    }

    /// Gets the virtual mouse wheel speed.
    pub fn mouse_wheel_speed_get(&mut self) -> Result<i8> {
        self.command_response_numerical("mouse.wheelSpeed")
    }

    /// Sets the virtual mouse wheel speed.
    pub fn mouse_wheel_speed_set(&mut self, speed: i8) -> Result<()> {
        self.command(&format!("mouse.wheelSpeed {}", speed))
    }

    /// Gets the virtual mouse wheel delay.
    pub fn mouse_wheel_delay_get(&mut self) -> Result<i8> {
        self.command_response_numerical("mouse.wheelDelay")
    }

    /// Sets the virtual mouse wheel delay.
    pub fn mouse_wheel_delay_set(&mut self, delay: i8) -> Result<()> {
        self.command(&format!("mouse.wheelDelay {}", delay))
    }

    /// Gets the virtual mouse speed limit.
    pub fn mouse_speed_limit_get(&mut self) -> Result<u8> {
        self.command_response_numerical("mouse.speedLimit")
    }

    /// Sets the virtual mouse speed limit.
    pub fn mouse_speed_limit_set(&mut self, limit: u8) -> Result<()> {
        self.command(&format!("mouse.speedLimit {}", limit))
    }

    /// Activate a certain layer remotely just by sending its order number. The layer number will start by 0 to address the first one and will end with 9 if we suppose a 10 layer list to address the last one. This does not affect the memory usage as the value is stored in RAM. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layeractivate
    pub fn layer_activate(&mut self, layer: u8) -> Result<()> {
        self.command(&format!("layer.activate {}", layer))
    }

    /// Deactivate the last layer that the keyboard switched to, this same function is the way the shift to layer key works on the keyboard. Just add the layer number at the end of the command to make the keyboard go back one layer. The layer number will start by 0 to address the first one and will end with 9 if we suppose a 10 layer list to address the last one. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerdeactivate
    pub fn layer_deactivate(&mut self) -> Result<()> {
        self.command("layer.deactivate")
    }

    /// Gets the current layer which is active. The layer number will start by 0 to address the first one and will end with 9 if we suppose a 10 layer list to address the last one. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerisactive
    pub fn layer_is_active_get(&mut self) -> Result<u8> {
        self.command_response_numerical("layer.isActive")
    }

    /// Switch to a certain layer remotely just by sending its order number. The layer number will start by 0 to address the first one and will end with 9 if we suppose a 10 layer list to address the last one. The difference between this command and the layer_activate alternative, is that the layer_activate adds to the layer switching history, but moveTo will erase that memory and return it to an array length 1 and holding the current layer the keyboard moved to. This does not affect the memory usage as the value is stored in RAM. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layermoveto
    pub fn layer_move_to(&mut self, layer: u8) -> Result<()> {
        self.command(&format!("layer.moveTo {}", layer))
    }

    /// Gets the status for up to 32 layers. It will return a Vec of bools with the state of each layer with the respective index. https://github.com/Dygmalab/Bazecor/blob/development/FOCUS_API.md#layerstate
    pub fn layer_state_get(&mut self) -> Result<Vec<bool>> {
        let response = self.command_response_string("layer.state")?;
        let parts = response.split_whitespace().collect::<Vec<&str>>();
        let nums = parts.iter().map(|&part| part == "1").collect();

        Ok(nums)
    }

    /// Gets the battery level of the left keyboard as a percentage.
    pub fn wireless_battery_level_left_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.left.level")
    }

    /// Gets the battery level of the right keyboard as a percentage.
    pub fn wireless_battery_level_right_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.right.level")
    }

    /// Gets the battery status of the left keyboard. Undocumented.
    pub fn wireless_battery_status_left_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.left.status")
    }

    /// Gets the battery status of the right keyboard. Undocumented.
    pub fn wireless_battery_status_right_get(&mut self) -> Result<u8> {
        self.command_response_numerical("wireless.battery.right.status")
    }

    /// Gets the battery saving mode state.
    pub fn wireless_battery_saving_mode_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.battery.savingMode")
    }

    /// Sets the battery saving mode state.
    pub fn wireless_battery_saving_mode_set(&mut self, state: bool) -> Result<()> {
        self.command(&format!("wireless.battery.savingMode {}", state as u8))
    }

    /// Gets the RF power level.
    pub fn wireless_rf_power_get(&mut self) -> Result<WirelessPowerMode> {
        self.command_response_numerical("wireless.rf.power")
    }

    /// Sets the RF power level.
    pub fn wireless_rf_power_set(&mut self, wireless_power_mode: WirelessPowerMode) -> Result<()> {
        self.command(&format!(
            "wireless.rf.power {}",
            wireless_power_mode.value()
        ))
    }

    /// Gets the RF channel hop state. Undocumented.
    pub fn wireless_rf_channel_hop_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.rf.channelHop")
    }

    /// Sets the RF channel hop state. Undocumented.
    pub fn wireless_rf_channel_hop_set(&mut self, state: bool) -> Result<()> {
        self.command(&format!("wireless.rf.channelHop {}", state as u8))
    }

    /// Gets the sync pairing state. Undocumented.
    pub fn wireless_rf_sync_pairing_get(&mut self) -> Result<bool> {
        self.command_response_bool("wireless.rf.syncPairing")
    }
}
