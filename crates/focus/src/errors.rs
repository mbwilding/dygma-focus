use std::fmt::Debug;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum FocusError {
    #[error("error enumerating serial ports: {0}")]
    SerialPortEnumerationError(#[source] serialport::Error),

    #[error("error connecting to serial port: {0}")]
    SerialPortOpenError(#[source] serialport::Error),

    #[error("error reading from serial port: {0}")]
    SerialPortReadError(#[source] std::io::Error),

    #[error("error writing to serial port: {0}")]
    SerialPortWriteError(#[source] std::io::Error),

    #[error("error flushing to serial port: {0}")]
    SerialPortFlushError(#[source] std::io::Error),

    #[error("error configuring serial port: {0}")]
    SerialPortConfigurationError(#[source] serialport::Error),

    #[error("response is empty")]
    EmptyResponseError,

    #[error("response does not contain exactly {expected} parts)")]
    PartCountError { expected: u8 },

    #[error("{label} beyond upper limit (max: {max}, provided: {provided})")]
    ValueAboveLimitError {
        label: &'static str,
        max: usize,
        provided: usize,
    },

    #[error("failed to convert response to UTF-8 string: {0}")]
    Utf8ConversionError(#[from] std::str::Utf8Error),

    #[error("string cannot be parse to bool: {string}")]
    ParseBoolError { string: String },

    #[error("failed to parse to int: {0}")]
    ParseIntError(#[from] std::num::ParseIntError),

    #[error("failed to parse to numerical value: {string}")]
    ParseNumericalError { string: String },

    #[error("failed to parse to numerical vec: {string}")]
    ParseNumericalVecError { string: String },

    #[error("chuck does not contain expected parts (actual: {actual}, expected: {expected}")]
    ChunkCountError { actual: usize, expected: usize },

    #[error("device not ready for upgrade, disconnect all sides, or if sides are connected press the top left key")]
    DeviceNotReadyError,

    #[error("no devices were detected")]
    NoDevicesDetectedError,

    #[error("side disconnected: {side:?}")]
    SideDisconnectedError { side: crate::enums::Side },
}
