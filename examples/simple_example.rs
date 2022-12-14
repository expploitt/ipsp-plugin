use ipsp_plugin::IpspManager;
use async_ctrlc::CtrlC;
use futures::StreamExt;


///
/// Simple example with name filter configuration
///
#[async_std::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let ipsp = IpspManager::new(Some(String::from("IPSP Device")));

    ipsp.start().await?;

    let signal = CtrlC::new().expect("Error creating CTRL+C signal handler");
    let mut _ctrl_c = signal.enumerate().take(1);

    _ctrl_c.next().await;
    ipsp.stop().await;

    Ok(())
}
