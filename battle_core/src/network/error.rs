use std::fmt::Display;

#[derive(Debug)]
pub enum NetworkError {
    NetworkError(String),
    ReceiveError(String),
    SendError(String),
}

impl From<zmq::Error> for NetworkError {
    fn from(error: zmq::Error) -> Self {
        Self::NetworkError(error.to_string())
    }
}

impl Display for NetworkError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NetworkError::NetworkError(message) => {
                f.write_str(&format!("NetworkError: {}", message))
            }
            NetworkError::ReceiveError(message) => {
                f.write_str(&format!("ReceiveError: {}", message))
            }
            NetworkError::SendError(message) => f.write_str(&format!("SendError: {}", message)),
        }
    }
}
