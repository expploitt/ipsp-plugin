use ipsp_plugin::IpspManager;
use async_ctrlc::CtrlC;
use futures::StreamExt;


///
/// Simple example with default configuration which will not filter
/// any type of device by name
/// 
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ipsp = IpspManager::default();

    ipsp.start().await?;

    let signal = CtrlC::new().expect("Error creating CTRL+C signal handler");
    let mut _ctrl_c = signal.enumerate().take(1);

    _ctrl_c.next().await;
    ipsp.stop().await;

    Ok(())
}
