use grpc_ring::Remote;
use std::env;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let next = env::var("NEXT")?;
    let mut remote = Remote::new(next).await?;
    remote.start_roll_call().await.unwrap();
    Ok(())
}
