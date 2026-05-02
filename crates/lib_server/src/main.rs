use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use lib_server::run_server;

#[tokio::main]
async fn main() -> Result<(), lib_server::ServerError> {
    let port = std::env::var("PORT")
        .ok()
        .and_then(|value| value.parse::<u16>().ok())
        .unwrap_or(8080);

    let bind_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), port);
    run_server(bind_addr).await
}
