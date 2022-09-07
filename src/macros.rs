#[macro_export]
macro_rules! connect_command {
    ($board_mac:expr, $_type:expr) => {
        format!("connect {} {}", $board_mac, $_type)
    };
}