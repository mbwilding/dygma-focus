pub mod devices;
pub mod structs;

use crate::devices::Device;
use crate::structs::Color;
use anyhow::{anyhow, bail, Result};
use devices::DEVICES;
use serialport::{SerialPort, SerialPortType};
use std::collections::HashMap;
use std::io::{Read, Write};
use std::time::Duration;
use tracing::{debug, error, trace};

#[derive(Default)]
pub struct Focus {
    port: Option<Box<dyn SerialPort>>,
}

impl Focus {
    pub fn find_all(&self) -> Result<Vec<Device>> {
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

    pub fn find_first(&self) -> Result<Device> {
        let devices = match self.find_all() {
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

    pub fn open_first(&mut self) -> Result<()> {
        self.open_via_device(self.find_all()?.first().ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
    }

    pub fn open_via_port(&mut self, port: &str) -> Result<()> {
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

    pub fn open_via_device(&mut self, device: &Device) -> Result<()> {
        self.open_via_port(&device.port)
    }

    fn command(&mut self, command: &str) -> Result<()> {
        trace!("Command TX: {}", command);

        if let Some(ref mut port) = self.port {
            port.write_all(format!("{}\n", command).as_bytes())?;

            Ok(())
        } else {
            bail!("Serial port is not open");
        }
    }

    fn command_response_string(&mut self, command: &str) -> Result<String> {
        self.command(command)?;

        let mut buffer = Vec::new();
        let eof_marker = b"\r\n.\r\n";

        if let Some(ref mut port) = self.port {
            loop {
                let prev_len = buffer.len();
                buffer.resize(prev_len + 1024, 0);
                match port.read(&mut buffer[prev_len..]) {
                    Ok(0) => continue,
                    Ok(size) => {
                        buffer.truncate(prev_len + size);

                        if buffer.ends_with(eof_marker) {
                            buffer.truncate(buffer.len() - eof_marker.len());
                            break;
                        }
                    }
                    Err(e) if e.kind() == std::io::ErrorKind::Interrupted => continue,
                    Err(e) => bail!("Error reading from serial port: {:?}", e),
                }
            }

            let start = buffer
                .iter()
                .position(|&b| !b.is_ascii_whitespace())
                .unwrap_or(0);
            let end = buffer
                .iter()
                .rposition(|&b| !b.is_ascii_whitespace())
                .map_or(0, |p| p + 1);
            let trimmed_buffer = &buffer[start..end];

            let response = String::from_utf8(trimmed_buffer.to_vec())
                .map_err(|e| anyhow!("Failed to convert response to UTF-8 string: {:?}", e))?;

            trace!("Command RX: {}", response);

            Ok(response)
        } else {
            Err(anyhow!("Serial port is not open"))
        }
    }

    fn command_response_vec_string(&mut self, command: &str) -> Result<Vec<String>> {
        Ok(self
            .command_response_string(command)?
            .lines()
            .map(|line| line.replace('\r', ""))
            .collect())
    }

    /// Get the version of the firmware.
    pub fn version_get(&mut self) -> Result<String> {
        self.command_response_string("version")
    }

    /// Gets the whole custom keymap stored in the keyboard. (Layers 0-9)
    pub fn keymap_custom_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.custom")
    }

    /// Sets the whole custom keymap stored in the keyboard. (Layers 0-9)
    pub fn keymap_custom_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("keymap.custom {}", data))
    }

    /// Gets the default keymap stored in the keyboard. (Layers -1 and -2)
    pub fn keymap_default_get(&mut self) -> Result<String> {
        self.command_response_string("keymap.default")
    }

    /// Sets the default keymap stored in the keyboard. (Layers -1 and -2)
    pub fn keymap_default_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("keymap.default {}", data))
    }

    /// Returns true or false depending on the user setting of hiding the default layers or not, it does not allow you to increment the number of available layers by start using the default ones, they are there so you can store a backup for two layers in your keyboard.
    pub fn keymap_only_custom_get(&mut self) -> Result<bool> {
        self.command_response_string("keymap.onlyCustom")
            .map(|response| response == "1")
    }

    /// Sets the user setting of hiding the default layers or not, it does not allow you to increment the number of available layers by start using the default ones, they are there so you can store a backup for two layers in your keyboard.
    pub fn keymap_only_custom_set(&mut self, state: bool) -> Result<()> {
        let value = match state {
            true => 1,
            false => 0,
        };
        self.command(&format!("keymap.onlyCustom {}", value))
    }

    /// Returns the default layer the keyboard will boot with.
    pub fn settings_default_layer_get(&mut self) -> Result<i8> {
        let response = self.command_response_string("settings.defaultLayer")?;
        response
            .parse::<i8>()
            .map_err(|e| anyhow!("Failed to parse response: {:?}", e))
    }

    /// Sets the default layer the keyboard will boot with.
    pub fn settings_default_layer_set(&mut self, layer: i8) -> Result<()> {
        self.command(&format!("settings.defaultLayer {}", layer))
    }

    /// Returns a boolean value that states true if all checks have been performed on the current settings and its upload was done in the intended way.
    pub fn settings_valid_get(&mut self) -> Result<bool> {
        let response = self.command_response_string("settings.valid?")?;
        response
            .parse()
            .map_err(|e| anyhow!("Failed to parse response: {:?}", e))
    }

    /// Gets the current settings version.
    pub fn settings_version_get(&mut self) -> Result<String> {
        self.command_response_string("settings.version")
    }

    /// Sets the current settings version.
    pub fn settings_version_set(&mut self, version: &str) -> Result<()> {
        self.command(&format!("settings.version {}", version))
    }

    /// Gets the current settings version.
    pub fn settings_crc_get(&mut self) -> Result<String> {
        self.command_response_string("settings.crc")
    }

    /// Gets the EEPROM's contents.
    pub fn eeprom_contents_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.contents")
    }

    /// Sets the EEPROM's contents.
    pub fn eeprom_contents_set(&mut self, data: &str) -> Result<()> {
        self.command(&format!("eeprom.contents {}", data))
    }

    /// Gets the EEPROM's free bytes.
    pub fn eeprom_free_get(&mut self) -> Result<String> {
        self.command_response_string("eeprom.free")
    }

    /// Gets the color of a specific LED.
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

    /// Sets the color of a specific LED.
    pub fn led_at_set(&mut self, led: u8, color: Color) -> Result<()> {
        self.command(&format!(
            "led.at {} {} {} {}",
            led, color.r, color.g, color.b
        ))
    }

    /// Gets the colors of specified LEDs.
    pub fn led_multiple_get(&mut self, leds: &[u8]) -> Result<HashMap<u8, Color>> {
        let response = self.command_response_vec_string(&format!(
            "led.getMultiple {}",
            leds.iter()
                .map(|led| led.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        ))?;

        if response.is_empty() {
            bail!("Empty response");
        }

        let mut led_colors = HashMap::new();

        for line in response {
            let parts = line.splitn(2, '#').collect::<Vec<&str>>();
            if parts.len() == 2 {
                let led_id = parts[0].trim().parse::<u8>()?;
                let color = parts[1].trim().parse::<Color>()?;
                led_colors.insert(led_id, color);
            } else {
                bail!("Invalid response");
            }
        }

        Ok(led_colors)
    }

    /// Sets the color of the specified LEDs.
    pub fn led_multiple_set(&mut self, color: Color, leds: &[u8]) -> Result<()> {
        self.command(&format!(
            "led.setMultiple {} {} {} {}",
            color.r,
            color.g,
            color.b,
            leds.iter()
                .map(|led| led.to_string())
                .collect::<Vec<String>>()
                .join(" ")
        ))
    }

    /// Sets the color of all the LEDs.
    pub fn led_all_set(&mut self, color: Color) -> Result<()> {
        self.command(&format!("led.setAll {} {} {}", color.r, color.g, color.b,))
    }

    /// Gets all of the available commands.
    pub fn help_get(&mut self) -> Result<Vec<String>> {
        self.command_response_vec_string("help")
    }

    /// Sets the layer, -1 to what you see in Bazecor. This does not write to the EEPROM.
    pub fn layer_move_to(&mut self, layer: u8) -> Result<()> {
        self.command(&format!("layer.moveTo {}", layer))
    }

    /// Gets the current layer, -1 to what you see in Bazecor.
    pub fn layer_is_active(&mut self) -> Result<String> {
        self.command_response_string("layer.isActive")
    }

    /// Gets the state of the layers, array index is -1 to what you see in Bazecor.
    pub fn layer_state(&mut self) -> Result<Vec<bool>> {
        let response = self.command_response_string("layer.state")?;
        let parts = response.split_whitespace().collect::<Vec<&str>>();
        let nums = parts.iter().map(|&part| part == "1").collect();

        Ok(nums)
    }
}
