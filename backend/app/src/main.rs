//! Main entry point for the BitNode Console backend server.

#[tokio::main]
async fn main() -> app::Result<()> {
    app::run().await
}
