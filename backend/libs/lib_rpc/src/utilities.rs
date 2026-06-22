// Re-export utilities service client from generated protobuf types
pub use crate::generated_protos::utilities_service_client::UtilitiesServiceClient;

pub use crate::generated_protos::utilities_service_server::{
    UtilitiesService, UtilitiesServiceServer,
};

pub use crate::generated_protos::{PingRequest, PingResponse};

#[derive(Default)]
pub struct MyUtilitiesService {}

// impl UtilitiesService for MyUtilitiesService {
//     fn ping(&self, request: PingRequest) -> PingResponse {
//         println!("Got a request from {:?}", request.remote_addr());

//         let reply: PingResponse = PingResponse {
//             message: "Pong...".to_string(),
//         };

//         Ok(Response::new(reply)) // Send back ping response
//     }
// }
