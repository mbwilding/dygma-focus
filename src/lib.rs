mod devices;

use crate::devices::Device;
use anyhow::{anyhow, Result};
use devices::DEVICES;
use serialport::{SerialPort, SerialPortType};
use std::io::Read;
use std::time::Duration;
use tracing::{debug, error};

pub struct Focus {
    port: Option<Box<dyn SerialPort>>,
}

impl Default for Focus {
    fn default() -> Self {
        Focus { port: None }
    }
}

impl Focus {
    pub fn find(&self) -> Result<Vec<Device>> {
        let ports = serialport::available_ports()?;
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

    pub fn open_first(&mut self) -> Result<()> {
        self.open_via_device(self.find()?.first().ok_or_else(|| {
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
            .timeout(Duration::from_millis(10));

        let port = port_settings.open().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        self.port = Some(port);

        Ok(())
    }

    pub fn open_via_device(&mut self, device: &Device) -> Result<()> {
        self.open_via_port(&device.port)
    }

    fn write_to_port(command: &str, port: &mut Box<dyn SerialPort>) -> Result<()> {
        port.write_all(command.as_bytes())?;
        port.write_all(b"\n")?;
        port.flush()?;

        Ok(())
    }

    pub fn command_no_response(&mut self, command: &str) -> Result<()> {
        if let Some(ref mut port) = self.port {
            Self::write_to_port(command, port)?;

            Ok(())
        } else {
            Err(anyhow!("Serial port is not open"))
        }
    }

    pub fn command_response(&mut self, command: &str) -> Result<String> {
        if let Some(ref mut port) = self.port {
            Self::write_to_port(command, port)?;

            let mut response = String::new();
            let bytes_read = port.read_to_string(&mut response)?;

            debug!("Read {} bytes: {}", bytes_read, response);

            Ok(response)
        } else {
            Err(anyhow!("Serial port is not open"))
        }
    }

    pub fn version(&mut self) -> Result<String> {
        self.command_response("version")
    }

    pub fn layer_move_to(&mut self, layer: u8) -> Result<()> {
        self.command_no_response(&format!("layer.moveTo {}", layer))
    }
}
