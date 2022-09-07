use std::fmt::Debug;
use std::fs::File;
use std::io::Write;
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

          
        let mut mod_command = Command::new("modprobe")
        .arg("bluetooth_6lowpan")
        .spawn()
        .expect("Error loading module");
        mod_command.wait().unwrap();


        let mut enable = File::create("/sys/kernel/debug/bluetooth/6lowpan_control").expect("Couldn't open file");
        let command = format!("1");
        enable.write_all(command.as_bytes()).unwrap();



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
                    if !device.connected && device.name.is_some(){
                        println!("Discovered device: {:?}", device);
                        /* TRY TO CONNECT */
                        let mac = &device.mac_address.to_string();
                        let _type = 2;

                        let mut file = File::create("/sys/kernel/debug/bluetooth/6lowpan_control").expect("Couldn't open file");
                        let command = format!("connect {} {}",mac,_type);
                        file.write_all(command.as_bytes()).unwrap();
                        
                      
                        //println!("{:?}", command);
                        
                        let mut addr_command = Command::new("ip")
                            .arg("address")
                            .arg("add")
                            .arg("2001:db8::2/64")
                            .arg("dev")
                            .arg("bt0")
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

