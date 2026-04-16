use std::io;

use CLI_ModBus_RTU_master_RUST::{run, terminal::enable_ansi_support};

#[tokio::main]
async fn main() -> io::Result<()> {
    enable_ansi_support();
    run().await
}
