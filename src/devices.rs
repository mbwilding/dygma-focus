#[derive(Debug)]
pub struct SupportedDevice {
    pub name: &'static str,
    pub vendor_id: u16,
    pub product_id: u16,
}

impl SupportedDevice {
    pub const fn new(name: &'static str, vendor_id: u16, product_id: u16) -> Self {
        SupportedDevice {
            name,
            vendor_id,
            product_id,
        }
    }
}

pub const DEVICES: [SupportedDevice; 4] = [
    SupportedDevice::new("Defy Wired", 0x35ef, 0x0010),
    SupportedDevice::new("Defy Wireless", 0x35ef, 0x0012),
    SupportedDevice::new("Raise ANSI", 0x1209, 0x2201),
    SupportedDevice::new("Raise ISO", 0x1209, 0x2201),
];

#[derive(Debug, Clone)]
pub struct Device {
    pub name: &'static str,
    pub port: String,
}
