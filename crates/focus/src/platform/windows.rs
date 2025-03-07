/// The Dygma Focus API.
#[derive(Debug)]
pub struct Focus {
    pub(crate) serial: serialport::COMPort,
    pub(crate) response_buffer: Vec<u8>,
}
