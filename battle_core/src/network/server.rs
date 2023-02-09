use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use std::thread;

use crate::{
    channel::Channel,
    message::{network::NetworkMessage, InputMessage, Message, OutputMessage},
};

use super::error::NetworkError;

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Server {
    rep_address: String,
    pub_address: String,
    send_sender: Sender<Vec<OutputMessage>>,
    send_receiver: Receiver<Vec<OutputMessage>>,
    receive_sender: Sender<Vec<InputMessage>>,
    receive_receiver: Receiver<Vec<InputMessage>>,
    error_sender: Sender<NetworkError>,
    error_receiver: Receiver<NetworkError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Envelope {
    id: u64,
    messages: Vec<OutputMessage>,
}

impl Envelope {
    pub fn new(id: u64, messages: Vec<OutputMessage>) -> Self {
        Self { id, messages }
    }
}

// TODO : When server/client is closing : end threads properly
impl Server {
    pub fn new(rep_address: String, pub_address: String, channel: &Channel) -> Self {
        Self {
            rep_address,
            pub_address,
            send_sender: channel.output_sender(),
            send_receiver: channel.output_receiver(),
            receive_sender: channel.input_sender(),
            receive_receiver: channel.input_receiver(),
            error_sender: channel.error_sender(),
            error_receiver: channel.error_receiver(),
        }
    }

    pub fn serve(&self) -> Result<(), NetworkError> {
        self.start_rep()?;
        self.start_pub()?;
        Ok(())
    }

    pub fn incoming_messages(&self) -> Vec<InputMessage> {
        let mut messages = vec![];
        while let Ok(messages_) = self.receive_receiver.try_recv() {
            messages.extend(messages_);
        }
        messages
    }

    pub fn errors(&self) -> Vec<NetworkError> {
        let mut errors = vec![];

        while let Ok(error) = self.error_receiver.try_recv() {
            errors.push(error);
        }

        errors
    }

    fn start_rep(&self) -> Result<(), NetworkError> {
        let thread_receive_sender = self.receive_sender.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_rep_address = self.rep_address.clone();

        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::REP)?;
        socket.bind(&server_rep_address)?;

        let ok = bincode::serialize(&Message::Network(NetworkMessage::Acknowledge)).unwrap();
        thread::spawn(move || {
            loop {
                // Receive client REQ messages bytes
                let messages_bytes = match socket.recv_bytes(0) {
                    Ok(message_bytes) => message_bytes,
                    Err(error) => {
                        thread_error_sender
                            .send(NetworkError::ReceiveError(format!("Error while receiving bytes : {}", error)))
                            .expect(&format!("Channel was closed when try to send receive communication error : {}", error));
                        continue;
                    }
                };

                // Decode received bytes into collection of messages
                let messages: Vec<InputMessage> = match bincode::deserialize(&messages_bytes) {
                    Ok(messages) => messages,
                    Err(error) => {
                        thread_error_sender
                            .send(NetworkError::ReceiveError(format!("Error while decoding received bytes : {}", error)))
                            .expect(&format!("Channel was closed when try to send receive communication error : {}", error));
                        continue;
                    }
                };

                // Send client expected acknowledgement
                socket.send(&ok, 0).unwrap_or_else(|e| {
                    thread_error_sender
                        .send(NetworkError::SendError(format!(
                            "Error while sending acknowledgement : {}",
                            e
                        )))
                        .expect(&format!(
                            "Channel was closed when try to send acknowledgement error : {}",
                            e
                        ));
                });

                // Send through channel the decoded messages
                thread_receive_sender.send(messages).expect(&format!(
                    "Channel was closed when try to send received messages"
                ));
            }
        });

        Ok(())
    }

    fn start_pub(&self) -> Result<(), NetworkError> {
        let thread_send_receiver = self.send_receiver.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_pub_address = self.pub_address.clone();

        let mut pub_counter: u64 = 0;
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::PUB)?;
        socket.bind(&server_pub_address)?;

        thread::spawn(move || loop {
            // Increment counter to permit client to know if some messages have been lost
            pub_counter += 1;

            // Retrieve messages to sent to clients
            let messages: Vec<OutputMessage> = thread_send_receiver.recv().expect(&format!(
                "Channel was closed when try to receive messages to send"
            ));

            // Prepare the data to send to clients
            let envelope = Envelope::new(pub_counter, messages);
            let messages_bytes = match bincode::serialize(&envelope) {
                Ok(messages_bytes) => messages_bytes,
                Err(error) => {
                    thread_error_sender
                        .send(NetworkError::SendError(format!(
                            "Error while encoding messages to send : {}",
                            error
                        )))
                        .expect(&format!(
                            "Channel was closed when try to send communication error : {}",
                            error
                        ));
                    continue;
                }
            };

            // Finally send messages to clients
            match socket.send(&messages_bytes, 0) {
                Err(error) => {
                    thread_error_sender
                        .send(NetworkError::SendError(format!(
                            "Error while sending messages : {}",
                            error
                        )))
                        .expect(&format!(
                            "Channel was closed when try to send send error : {}",
                            error
                        ));
                }
                _ => {}
            };
        });

        Ok(())
    }

    pub fn send(&self, messages: Vec<OutputMessage>) {
        self.send_sender.send(messages).unwrap();
    }
}
