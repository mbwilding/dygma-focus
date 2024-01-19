#[cfg(all(feature = "async", feature = "sync"))]
compile_error!(
    "`async` and `sync` features are mutually exclusive and cannot be enabled at the same time"
);

use crate::hardware::Device;
use anyhow::{anyhow, bail, Result};
use std::str;
use std::time::Duration;
use tracing::{error, trace};

#[cfg(feature = "async")]
use tokio_serial::{SerialPort, SerialPortBuilderExt, SerialPortType, SerialStream};

#[cfg(feature = "sync")]
use serialport::{SerialPort, SerialPortType};

pub mod api;
mod api_async;
mod api_sync;
pub mod color;
pub mod enums;
pub mod hardware;
pub(crate) mod helpers;
pub mod prelude;
pub mod settings;

pub const MAX_LAYERS: u8 = 10 - 1;

/// The Dygma Focus API.
#[cfg(feature = "async")]
#[derive(Debug)]
pub struct Focus {
    pub(crate) serial: tokio::sync::Mutex<SerialStream>,
    pub(crate) response_buffer: Vec<u8>,
}

#[cfg(feature = "sync")]
#[derive(Debug)]
pub struct Focus {
    pub(crate) serial: std::sync::Mutex<Box<dyn SerialPort>>,
    pub(crate) response_buffer: Vec<u8>,
}

/// Constructors
impl Focus {
    /// Find all supported devices.
    pub fn find_all_devices() -> Result<Vec<Device>> {
        #[cfg(feature = "async")]
        let available = tokio_serial::available_ports();

        #[cfg(feature = "sync")]
        let available = serialport::available_ports();

        let ports = match available {
            Ok(ports) => ports,
            Err(e) => {
                let err_msg = format!("Failed to enumerate serial ports: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

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
    pub fn find_first_device() -> Result<Device> {
        let devices = match Self::find_all_devices() {
            Ok(devices) => devices,
            Err(e) => {
                let err_msg = format!("No device found: {:?}", e);
                error!("{}", err_msg);
                bail!(err_msg)
            }
        };

        let device = devices.into_iter().nth(0).ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        Ok(device)
    }

    /// Creates a new instance of the Focus API, connecting to the device via the named serial port.
    #[cfg(feature = "async")]
    pub fn new_via_port(port: &str) -> Result<Self> {
        let port_settings = tokio_serial::new(port, 115_200)
            .data_bits(tokio_serial::DataBits::Eight)
            .flow_control(tokio_serial::FlowControl::None)
            .parity(tokio_serial::Parity::None)
            .stop_bits(tokio_serial::StopBits::One)
            .timeout(Duration::from_secs(5));

        let mut serial = port_settings.open_native_async().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        serial.write_data_terminal_ready(true)?;

        #[cfg(unix)]
        serial.set_exclusive(false)?;

        Ok(Self {
            serial: tokio::sync::Mutex::new(serial),
            response_buffer: Vec::with_capacity(1_024 * 8),
        })
    }

    /// Creates a new instance of the Focus API, connecting to the device via the named serial port.
    #[cfg(feature = "sync")]
    pub fn new_via_port(port: &str) -> Result<Self> {
        let port_settings = serialport::new(port, 115_200)
            .data_bits(serialport::DataBits::Eight)
            .flow_control(serialport::FlowControl::None)
            .parity(serialport::Parity::None)
            .stop_bits(serialport::StopBits::One)
            .timeout(Duration::from_secs(5));

        let mut serial = port_settings.open().map_err(|e| {
            let err_msg = format!("Failed to open serial port: {} ({:?})", &port, e);
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?;

        serial.write_data_terminal_ready(true)?;

        #[cfg(unix)]
        serial.set_exclusive(false)?;

        Ok(Self {
            serial: std::sync::Mutex::new(serial),
            response_buffer: Vec::with_capacity(1_024 * 8),
        })
    }

    /// Creates a new instance of the Focus API, connecting to the device via a reference to the device struct.
    pub fn new_via_device(device: &Device) -> Result<Self> {
        Self::new_via_port(&device.serial_port)
    }

    /// Creates a new instance of the Focus API, connecting to the device via first available device.
    pub fn new_first_available() -> Result<Self> {
        Self::new_via_device(Self::find_all_devices()?.first().ok_or_else(|| {
            let err_msg = "No supported devices found";
            error!("{}", err_msg);
            anyhow!(err_msg)
        })?)
    }
}
