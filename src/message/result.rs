use std::fmt::Display;

pub enum OCError {
    NetworkError(String),
    ReceiveError(String),
    SendError(String),
}

impl From<zmq::Error> for OCError {
    fn from(error: zmq::Error) -> Self {
        Self::NetworkError(error.to_string())
    }
}

impl Display for OCError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(&format!("OCError : {}", self))
    }
}
