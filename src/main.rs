use std::io;

use rust_cli_modbus_master::{run, terminal::enable_ansi_support};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_ansi_support();
    run().await
}
