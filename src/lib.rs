use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;
use bluez_async::{BluetoothSession, DiscoveryFilter};

mod device;
mod macros;

const SCANNING_SLOT: Duration = Duration::from_secs(5);

#[derive(Clone)]
pub struct IpspManager {
    name_filter: Option<String>,
    devices: Vec<device::Device>,
    flag: Arc<AtomicBool>,
}

impl Drop for IpspManager {
    fn drop(&mut self) {
        async_std::task::block_on(self.stop());
    }
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

    pub async fn start(&self) -> (async_std::channel::Sender<()>, async_std::task::JoinHandle<()>) {
        let (sender, receiver) = async_std::channel::bounded(1);

        let ipsp = self.clone();
        let handle = async_std::task::spawn_blocking(move || {
            async_std::task::block_on(async { ipsp.run(receiver).await })
        });

        (sender, handle)
    }

    pub async fn stop(&self) {
        self.flag.store(false, Ordering::Relaxed);
    }

    async fn run(&self, recv: async_std::channel::Receiver<()>) {
        println!("IPSP Plugin running...");
        /* SETUP 6LOWPAN CONFIG*/

        /* INIT SESSION*/
        let (bluez_handle, session) = bluez_async::BluetoothSession::new().await.unwrap();


        while self.flag.load(Ordering::Relaxed) {
            self.discover(&session).await;

            if let Ok(devices) = session.get_devices().await {
                for device in devices {
                    if !device.connected {
                        println!("Discovered device: {:?}", device);
                        /* TRY TO CONNECT */
                        let mac = &device.mac_address.to_string();
                        let _type = 2;

                        let command = connect_command!(mac, _type);
                        let mut command = Command::new(command)
                            .spawn()
                            .expect("Error connecting device");

                        command.wait().unwrap();

                        let mut addr_command = Command::new("ip address add 2001:db8::2/64 dev bt0")
                            .spawn()
                            .expect("Error configuring ip address");

                        addr_command.wait().unwrap();
                    }
                }
            } else {
                println!("Devices not discovered!");
            }
            /* ADD NEW TASK FOR NEW DEVICE */
        }
    }

    pub async fn discover(&self, session: &BluetoothSession) {
        let mut filter = DiscoveryFilter::default();
        filter.pattern = self.name_filter.clone();

        session.start_discovery_with_filter(&filter).await.unwrap();
        async_std::task::sleep(SCANNING_SLOT).await;
        session.stop_discovery().await.unwrap();
    }
}

