use std::error::Error;
use std::fs::File;
use std::io::Write;
use std::process::Command;
use std::time::Duration;

pub const BLUETHOOH_6LOWPAN_CONTROL_FILE: &str = "/sys/kernel/debug/bluetooth/6lowpan_control";
pub const SCANNING_SLOT: Duration = Duration::from_secs(5);

pub fn enable_bluetooth_6lowpan() -> Result<(), Box<dyn Error>> {
    let mut mod_command = Command::new("modprobe")
        .arg("bluetooth_6lowpan")
        .spawn()?;

    mod_command.wait()?;

    log::info!("Modprobe Bluetooth 6LowPAN enabled");

    Ok(())
}

pub fn enable_6lowpan_control() -> Result<(), Box<dyn Error>> {
    let mut enable = File::create("/sys/kernel/debug/bluetooth/6lowpan_control")?;
    enable.write_all(format!("1").as_bytes())?;

    log::info!("Modprobe Bluetooth 6LowPAN enabled");

    Ok(())
}