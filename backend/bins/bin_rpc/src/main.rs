//! Main entry point for the BitNode Console RPC-only server.

#[tokio::main]
async fn main() -> bin_rpc::Result<()> {
    bin_rpc::run().await
}
