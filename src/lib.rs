use std::error;
use std::fmt::{Debug, Display, Formatter};
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use bluez_async::{BluetoothSession, DeviceInfo, DiscoveryFilter};

use crate::device::Device;

mod device;
mod macros;
mod config;

type Result<T> = std::result::Result<T, Box<dyn error::Error>>;

#[derive(Clone, Debug)]
pub struct IpspManager {
    name_filter: Option<String>,
    devices: Vec<device::Device>,
    flag: Arc<AtomicBool>,
}

impl Default for IpspManager {
    fn default() -> Self {
        Self {
            name_filter: Some(String::from("GTI IPSP")),
            devices: vec![],
            flag: Arc::new(AtomicBool::new(true)),
        }
    }
}

impl IpspManager {
    pub fn new(filter: Option<String>) -> Self {
        Self {
            name_filter: filter,
            devices: vec![],
            flag: Arc::new(AtomicBool::new(true)),
        }
    }

    pub async fn start(&self) -> Result<()> {
        env_logger::init();

        config::enable_bluetooth_6lowpan()?;
        config::enable_6lowpan_control()?;

        let mut ipsp = self.clone();
        let _handle = async_std::task::spawn_blocking(move || {
            async_std::task::block_on(async { ipsp.run().await.unwrap() })
        });

        Ok(())
    }

    pub async fn stop(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }

    async fn run(&mut self) -> Result<()> {
        log::info!("IPSP Plugin running...");

        /* INIT SESSION*/
        let (_bluez_handle, session) = BluetoothSession::new().await.unwrap();


        while self.flag.load(Ordering::Relaxed) {
            self.discover(&session).await?;

            if let Ok(devices) = session.get_devices().await {

                // If we filter, we need to remove None name devices
                let devices = self.filter_devices_by_name(devices);

                for device in devices {
                    if !device.connected {
                        log::info!("\nDiscovered device -> {:?} \n", device);

                        /* TRY TO CONNECT */
                        let mac = device.mac_address.clone().to_string();
                        let type_ = "2";

                        match self.connect(&mac, type_) {
                            Ok(_) => {
                                let new_device = Device::new(
                                    device.mac_address.to_string(),
                                    device.name.unwrap_or(String::from("None")));

                                self.devices.push(new_device);
                            }
                            Err(e) => { log::error!("Error trying to connect to device {}: {}", mac, e.to_string()); }
                        };
                    }
                }
            } else {
                println!("Devices not available");
            }
        }

        Ok(())
    }

    pub fn connect(&mut self, mac: &str, type_: &str) -> Result<()> {
        let mut file = File::create(config::BLUETHOOH_6LOWPAN_CONTROL_FILE)?;
        file.write_all(connect_command!(mac, type_).as_bytes())?;

        let mut addr_command = Command::new("ip")
            .arg("address")
            .arg("add")
            .arg("2001:db8::2/64")
            .arg("dev")
            .arg("bt0")
            .spawn()
            .expect("Error configuring ip address");

        addr_command.wait().unwrap();

        log::info!("Device {} connected to system", &mac);

        Ok(())
    }

    pub async fn discover(&self, session: &BluetoothSession) -> Result<()> {
        let mut filter = DiscoveryFilter::default();
        filter.pattern = self.name_filter.clone();

        log::info!("Discovering devices...");

        session.start_discovery_with_filter(&filter).await?;
        async_std::task::sleep(config::SCANNING_SLOT).await;
        session.stop_discovery().await?;

        Ok(())
    }

    pub fn filter_devices_by_name(&self, mut devices: Vec<DeviceInfo>) -> Vec<DeviceInfo> {
        if self.name_filter.is_some() {
            return devices.iter_mut().filter(|x| x.name.is_some()).map(|x| x.clone()).collect();
        }

        devices
    }
}

impl Display for IpspManager {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "Error: ")
    }
}

impl error::Error for IpspManager {}

impl Drop for IpspManager {
    fn drop(&mut self) {
        async_std::task::block_on(self.stop());
    }
}