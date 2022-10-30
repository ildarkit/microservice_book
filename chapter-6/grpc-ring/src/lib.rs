pub mod grpc {
    tonic::include_proto!("ringproto");
}
mod error;

use std::time::Duration;
use tonic::{Request, Response, Status, transport::{Channel, Endpoint}};
use crate::error::RingGrpcError;
use crate::grpc::ring_client::RingClient;
use crate::grpc::Empty;

pub struct Remote<Channel>  {
    client: RingClient<Channel>,
}

impl Remote<Channel> { 
    pub async fn new(addr: String) -> Result<Self, Box<dyn std::error::Error>> {
        let channel = Endpoint::new(addr)?
            .timeout(Duration::from_secs(3))
            .connect_timeout(Duration::from_secs(10))
            .connect_lazy();
        let client = RingClient::new(channel);
        Ok(Self {
            client,
        })
    }

    pub async fn start_roll_call(&mut self) -> Result<Empty, RingGrpcError> {
        let response = self.client
            .start_roll_call(Request::new(Empty {}))
            .await;
        self.get_from_response(response)
    }

    pub async fn mark_itself(&mut self) -> Result<Empty, RingGrpcError> {
        let response = self.client
            .mark_itself(Request::new(Empty {}))
            .await;
        self.get_from_response(response)
    }

    fn get_from_response(&self, response: Result<Response<Empty>, Status>)
        -> Result<Empty, RingGrpcError>
    {
        match response {
            Ok(resp) => Ok(resp.into_inner()),
            Err(status) => {
                log::error!("Error response with status {}", status);
                Err(
                    RingGrpcError::new(
                        format!("Internal error: {}", status).as_str()
                        )
                    )
            },
        }
    }
}
