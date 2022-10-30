use grpc_ring::grpc::Empty;
use grpc_ring::grpc::ring_server::{Ring, RingServer};
use grpc_ring::Remote;
use log::{debug, trace, error, info};
use std::env;
use tonic::{transport::Server, Request, Response, Status};
use tokio::{self, sync::mpsc::{self, Sender, Receiver}};


#[derive(Debug)]
enum Action {
    StartRollCall,
    MarkItself,
}

#[derive(Debug)]
struct RingService {
    sender: Sender<Action>,
}

impl RingService {
    fn new(sender: Sender<Action>) -> Self {
        Self {
            sender,
        }
    }

    async fn send_action(&self, action: Action) -> Result<Response<Empty>, Status> {
        self.sender.send(action).await.unwrap();
        Ok(Response::new(Empty {}))
    }
}

#[tonic::async_trait]
impl Ring for RingService {
    async fn start_roll_call(&self, _: Request<Empty>)
        ->  Result<Response<Empty>, Status>
    {
        trace!("START_ROLL_CALL");
        self.send_action(Action::StartRollCall).await
    }

    async fn mark_itself(&self, _: Request<Empty>) 
        -> Result<Response<Empty>, Status>
    {
        trace!("MARK_INSELF");
        self.send_action(Action::MarkItself).await
    }
}

async fn worker_loop(mut receiver: Receiver<Action>)
    -> Result<(), Box<dyn std::error::Error>>
{
    let next = match env::var("NEXT") {
        Ok(val) => val,
        Err(e) => {
            error!("$NEXT is not set: {}", e.to_string());
            return Err(Box::new(e))
        }
    };
    let mut remote = Remote::new(next).await?;
    let mut in_roll_call = false;
    loop {
        match receiver.recv().await {
            Some(Action::StartRollCall) => {
                if !in_roll_call {
                    if remote.start_roll_call().await.is_ok() {
                        debug!("ON");
                        in_roll_call = true;
                    }
                } else {
                    if remote.mark_itself().await.is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                }
            },
            Some(Action::MarkItself) => {
                if in_roll_call {
                    if remote.mark_itself().await.is_ok() {
                        debug!("OFF");
                        in_roll_call = false;
                    }
                } else {
                    debug!("SKIP");
                }
            },
            None => break,
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();
    let (tx, rx) = mpsc::channel(4);
    let addr = env::var("ADDRESS")?.parse()?;
    let ring_service = RingService::new(tx);
    let svc = RingServer::new(ring_service);

    info!("Worker is running");
    tokio::spawn(async move {
        worker_loop(rx).await.unwrap();
    });

    info!("Server is running"); 
    Server::builder()
        .add_service(svc)
        .serve(addr)
        .await?; 

    Ok(())
}
