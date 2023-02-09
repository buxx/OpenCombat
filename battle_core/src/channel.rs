use crossbeam_channel::{unbounded, Receiver, Sender};

use crate::{
    message::{InputMessage, OutputMessage},
    network::error::NetworkError,
};

#[derive(Clone)]
pub struct Channel {
    input_sender: Sender<Vec<InputMessage>>,
    input_receiver: Receiver<Vec<InputMessage>>,
    output_sender: Sender<Vec<OutputMessage>>,
    output_receiver: Receiver<Vec<OutputMessage>>,
    error_sender: Sender<NetworkError>,
    error_receiver: Receiver<NetworkError>,
}

impl Channel {
    pub fn new() -> Self {
        let (input_sender, input_receiver) = unbounded();
        let (output_sender, output_receiver) = unbounded();
        let (error_sender, error_receiver) = unbounded();

        Self {
            input_sender,
            input_receiver,
            output_sender,
            output_receiver,
            error_sender,
            error_receiver,
        }
    }

    pub fn input_sender(&self) -> Sender<Vec<InputMessage>> {
        self.input_sender.clone()
    }

    pub fn input_receiver(&self) -> Receiver<Vec<InputMessage>> {
        self.input_receiver.clone()
    }

    pub fn output_sender(&self) -> Sender<Vec<OutputMessage>> {
        self.output_sender.clone()
    }

    pub fn output_receiver(&self) -> Receiver<Vec<OutputMessage>> {
        self.output_receiver.clone()
    }

    pub fn error_sender(&self) -> Sender<NetworkError> {
        self.error_sender.clone()
    }

    pub fn error_receiver(&self) -> Receiver<NetworkError> {
        self.error_receiver.clone()
    }
}
