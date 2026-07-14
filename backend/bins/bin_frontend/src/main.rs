//! Main entry point for the BitNode Console web-frontend-only server.

#[tokio::main]
async fn main() -> bin_frontend::Result<()> {
    bin_frontend::run().await
}
