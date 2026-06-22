//! Main entry point for the BitNode Console backend server.

#[tokio::main]
async fn main() -> application::Result<()> {
    application::run().await
}
