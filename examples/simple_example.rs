

use ipsp_plugin::IpspManager;
use async_ctrlc::CtrlC;
use futures::StreamExt;

#[async_std::main]
async fn main() {

    env_logger::init();
    let ipsp = IpspManager::new(None);

    let (a, handle) = ipsp.start().await;

    let signal = CtrlC::new().expect("Error creating CTRL+C signal handler");
    let mut _ctrl_c = signal.enumerate().take(1);

    _ctrl_c.next().await;
    ipsp.stop().await;
}
