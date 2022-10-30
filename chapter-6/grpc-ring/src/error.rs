use std::fmt;

#[derive(Debug)]
pub struct RingGrpcError {
    message: String,
}

impl RingGrpcError {
    pub fn new(msg: &str) -> Self {
        RingGrpcError {
            message: msg.to_string(),
        }
    }
}

impl fmt::Display for RingGrpcError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message)
    }
} 
