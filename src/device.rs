#[derive(Clone, Debug)]
pub struct Device {
    name: String,
    mac_addr: String,
}

impl Device {
    pub fn new(name: String, mac_addr: String) -> Self {
        Self {
            name,
            mac_addr,
        }
    }
}