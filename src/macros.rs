#[macro_export]
macro_rules! connect_command {
    ($board_mac:expr, $_type:expr) => {
        format!("connect {} {} > /sys/kernel/debug/bluetooth/6lowpan_control", $board_mac, $_type)
    };
}