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
        f.write_str(&format!("OCError : {}", self))
    }
}
