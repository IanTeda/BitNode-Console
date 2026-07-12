//! Main entry point for the BitNode Console RPC-only server.

#[tokio::main]
async fn main() -> bin_backend::Result<()> {
    bin_backend::run().await
}
