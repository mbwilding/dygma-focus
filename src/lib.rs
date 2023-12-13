mod devices;

use crate::devices::Device;
use anyhow::{anyhow, Result};
use devices::DEVICES;
use serialport::{SerialPort, SerialPortType};
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

    pub fn open_single(&mut self) -> Result<()> {
        self.open_specific(self.find()?.first().ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
    }

    pub fn open_specific(&mut self, device: &Device) -> Result<()> {
        let port_settings = serialport::new(&device.port, 115_200)
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_millis(10));

        let port = port_settings.open().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &device.port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        self.port = Some(port);

        Ok(())
    }

    pub fn command(&mut self, command: &str) -> Result<()> {
        if let Some(ref mut port) = self.port {
            port.write_all(command.as_bytes())?;
            port.write_all(b"\n")?;
            port.flush()?;

            Ok(())
        } else {
            Err(anyhow!("Serial port is not open"))
        }
    }
}
