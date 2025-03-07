use crate::hardware::Device;
use errors::FocusError;
use log::trace;
use serialport::{SerialPort, SerialPortInfo, SerialPortType};
use std::str;
use std::time::Duration;

#[cfg(windows)]
use serialport::COMPort;

#[cfg(not(windows))]
use serialport::TTYPort;

pub mod api;
pub mod color;
pub mod enums;
pub mod errors;
pub mod hardware;
pub mod helpers;
pub mod prelude;
pub mod settings;

pub const MAX_LAYERS: u8 = 10 - 1;

/// The Dygma Focus API.
#[derive(Debug)]
pub struct Focus {
    #[cfg(windows)]
    pub(crate) serial: COMPort,
    #[cfg(not(windows))]
    pub(crate) serial: TTYPort,
    pub(crate) response_buffer: Vec<u8>,
}

/// Constructors
impl Focus {
    /// Find all supported devices.
    pub fn find_all_devices() -> Result<Vec<Device>, FocusError> {
        let ports =
            serialport::available_ports().map_err(FocusError::SerialPortEnumerationError)?;
        Self::collect_devices(ports)
    }

    /// Detects and returns the connected devices
    fn collect_devices(ports: Vec<SerialPortInfo>) -> Result<Vec<Device>, FocusError> {
        trace!("Available serial ports: {:?}", ports);
        let devices: Vec<Device> = ports
            .into_iter()
            .filter_map(|port| match &port.port_type {
                SerialPortType::UsbPort(info) => {
                    let matching_devices: Vec<Device> =
                        hardware::types::hardware_physical::DEVICES_PHYSICAL
                            .iter()
                            .filter_map(|device| {
                                if device.usb.vendor_id == info.vid
                                    && device.usb.product_id == info.pid
                                {
                                    Some(Device {
                                        hardware: device.to_owned(),
                                        serial_port: port.port_name.to_owned(),
                                    })
                                } else {
                                    None
                                }
                            })
                            .collect();
                    if matching_devices.is_empty() {
                        None
                    } else {
                        Some(matching_devices)
                    }
                }
                _ => None,
            })
            .flatten()
            .collect();
        trace!("Found devices: {:?}", devices);
        Ok(devices)
    }

    /// Find the first supported device.
    pub fn find_first_device() -> Result<Device, FocusError> {
        let devices = Self::find_all_devices()?;
        let device = devices
            .into_iter()
            .nth(0)
            .ok_or(FocusError::NoDevicesDetectedError)?;
        Ok(device)
    }

    /// Creates a new instance of the Focus API, connecting to the device via the named serial port.
    pub fn new_via_port(port: &str) -> Result<Self, FocusError> {
        let port_settings = serialport::new(port, 115_200)
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_secs(5));
        let mut serial = port_settings
            .open_native()
            .map_err(FocusError::SerialPortOpenError)?;
        serial
            .write_data_terminal_ready(true)
            .map_err(FocusError::SerialPortConfigurationError)?;
        #[cfg(unix)]
        serial
            .set_exclusive(false)
            .map_err(FocusError::SerialPortConfigurationError)?;
        Ok(Self {
            serial,
            response_buffer: Vec::with_capacity(1_024 * 8),
        })
    }

    /// Creates a new instance of the Focus API, connecting to the device via a reference to the device struct.
    pub fn new_via_device(device: &Device) -> Result<Self, FocusError> {
        Self::new_via_port(&device.serial_port)
    }

    /// Creates a new instance of the Focus API, connecting to the device via first available device.
    pub fn new_first_available() -> Result<Self, FocusError> {
        Self::new_via_device(
            Self::find_all_devices()?
                .first()
                .ok_or(FocusError::NoDevicesDetectedError)?,
        )
    }
}
