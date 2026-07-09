//! Main entry point for the BitNode Console full server (web + RPC).

#[tokio::main]
async fn main() -> bin_console::Result<()> {
    bin_console::run().await
}
