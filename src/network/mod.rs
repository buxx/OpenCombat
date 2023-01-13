use crossbeam_channel::{unbounded, Receiver, Sender};
use ggez::GameResult;
use serde::{Deserialize, Serialize};

use std::thread;

use crate::config::Config;
use crate::message::{result::*, *};

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Network {
    config: Config,
    send_sender: Sender<Vec<Message>>,
    send_receiver: Receiver<Vec<Message>>,
    receive_sender: Sender<Vec<Message>>,
    receive_receiver: Receiver<Vec<Message>>,
    error_sender: Sender<OCError>,
    error_receiver: Receiver<OCError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Envelope {
    id: u64,
    messages: Vec<Message>,
}

impl Envelope {
    pub fn new(id: u64, messages: Vec<Message>) -> Self {
        Self { id, messages }
    }
}

// TODO : When server/client is closing : end threads properly
impl Network {
    pub fn new(config: Config) -> GameResult<Self> {
        let (send_sender, send_receiver): (Sender<Vec<Message>>, Receiver<Vec<Message>>) =
            unbounded();
        let (receive_sender, receive_receiver): (Sender<Vec<Message>>, Receiver<Vec<Message>>) =
            unbounded();
        let (error_sender, error_receiver): (Sender<OCError>, Receiver<OCError>) = unbounded();

        Ok(Self {
            config,
            send_sender,
            send_receiver,
            receive_sender,
            receive_receiver,
            error_sender,
            error_receiver,
        })
    }

    pub fn init(&mut self) -> Result<(), OCError> {
        match self.config.network_mode() {
            crate::NetworkMode::Server => {
                self.start_rep()?;
                self.start_pub()?
            }
            crate::NetworkMode::Client => {
                self.start_req()?;
                self.start_sub()?;
            }
        };
        Ok(())
    }

    /// Return received messages from remote :
    ///  - As server : messages from clients
    ///  - As client : messages from server
    pub fn incoming_messages(&self) -> Vec<Message> {
        let mut messages = vec![];
        while let Ok(messages_) = self.receive_receiver.try_recv() {
            messages.extend(messages_);
        }
        messages
    }

    pub fn errors(&self) -> Vec<OCError> {
        let mut errors = vec![];

        while let Ok(error) = self.error_receiver.try_recv() {
            errors.push(error);
        }

        errors
    }

    fn start_rep(&self) -> Result<(), OCError> {
        let thread_receive_sender = self.receive_sender.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_rep_address = self.config.server_rep_address().clone();

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
                            .send(OCError::ReceiveError(format!("Error while receiving bytes : {}", error)))
                            .expect(&format!("Channel was closed when try to send receive communication error : {}", error));
                        continue;
                    }
                };

                // Decode received bytes into collection of messages
                let messages: Vec<Message> = match bincode::deserialize(&messages_bytes) {
                    Ok(messages) => messages,
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::ReceiveError(format!("Error while decoding received bytes : {}", error)))
                            .expect(&format!("Channel was closed when try to send receive communication error : {}", error));
                        continue;
                    }
                };

                // Send client expected acknowledgement
                socket.send(&ok, 0).unwrap_or_else(|e| {
                    thread_error_sender
                        .send(OCError::SendError(format!(
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

    fn start_req(&self) -> Result<(), OCError> {
        let thread_send_receiver = self.send_receiver.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_rep_address = self.config.server_rep_address().clone();

        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::REQ)?;
        socket.connect(&server_rep_address)?;

        thread::spawn(move || {
            loop {
                // Wait messages to send
                let messages: Vec<Message> = thread_send_receiver.recv().expect(&format!(
                    "Channel was closed when try to receive messages to send"
                ));

                // Encode messages to send
                let messages_bytes = match bincode::serialize(&messages) {
                    Ok(messages_bytes) => messages_bytes,
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::SendError(format!(
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

                // Send messages to server
                match socket.send(messages_bytes, 0) {
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::SendError(format!(
                                "Error while sending messages : {}",
                                error
                            )))
                            .expect(&format!(
                                "Channel was closed when try to send messages error : {}",
                                error
                            ));

                        // Don't expect a response if send error
                        continue;
                    }
                    _ => {}
                };

                let _response = match socket.recv_bytes(0) {
                    Ok(response) => response,
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::SendError(format!(
                                "Error while receiving server REP : {}",
                                error
                            )))
                            .expect(&format!(
                                "Channel was closed when try to send messages error : {}",
                                error
                            ));

                        // Don't expect a response if send error
                        continue;
                    }
                };

                // Don't check the response content. The server ACK is only required here.
            }
        });

        Ok(())
    }

    fn start_pub(&self) -> Result<(), OCError> {
        let thread_send_receiver = self.send_receiver.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_pub_address = self.config.server_pub_address().clone();

        let mut pub_counter: u64 = 0;
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::PUB)?;
        socket.bind(&server_pub_address)?;

        thread::spawn(move || loop {
            // Increment counter to permit client to know if some messages have been lost
            pub_counter += 1;

            // Retrieve messages to sent to clients
            let messages = thread_send_receiver.recv().expect(&format!(
                "Channel was closed when try to receive messages to send"
            ));

            // Prepare the data to send to clients
            let envelope = Envelope::new(pub_counter, messages);
            let messages_bytes = match bincode::serialize(&envelope) {
                Ok(messages_bytes) => messages_bytes,
                Err(error) => {
                    thread_error_sender
                        .send(OCError::SendError(format!(
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
                        .send(OCError::SendError(format!(
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

    fn start_sub(&self) -> Result<(), OCError> {
        let thread_receive_sender = self.receive_sender.clone();
        let thread_send_sender = self.send_sender.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_pub_address = self.config.server_pub_address().clone();

        let mut last_counter: u64 = 0;
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::SUB)?;
        socket.connect(&server_pub_address)?;
        // TODO : subscribe with client ID and ALL (to receive all messages except global sync of other clients)
        socket.set_subscribe(b"")?;

        thread::spawn(move || {
            loop {
                // Receive server messages
                let envelope_bytes = match socket.recv_bytes(0) {
                    Ok(envelope_bytes) => envelope_bytes,
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::SendError(format!(
                                "Error while receiving server messages : {}",
                                error
                            )))
                            .expect(&format!(
                                "Channel was closed when try to send receive error : {}",
                                error
                            ));

                        // Waiting again if receive error
                        continue;
                    }
                };

                // Decode received messages
                let envelope: Envelope = match bincode::deserialize(&envelope_bytes) {
                    Ok(envelope) => envelope,
                    Err(error) => {
                        thread_error_sender
                            .send(OCError::ReceiveError(format!("Error while decoding received messages bytes : {}", error)))
                            .expect(&format!("Channel was closed when try to send receive communication error : {}", error));
                        continue;
                    }
                };

                // Send through channel the decoded messages
                thread_receive_sender
                    .send(envelope.messages)
                    .expect(&format!(
                        "Channel was closed when try to send received messages"
                    ));

                // Check no message(s) was lost, if yes, require sync from server
                if last_counter != 0 && last_counter + 1 != envelope.id {
                    println!("WARNING :: Network :: message(s) lost, require global Sync");
                    thread_send_sender
                        .send(vec![Message::Network(NetworkMessage::RequireCompleteSync)])
                        .expect(&format!(
                            "Channel was closed when try to send server sync requirement"
                        ));
                }

                // Update the last counter
                last_counter = envelope.id;
            }
        });

        Ok(())
    }

    pub fn send(&self, messages: Vec<Message>) {
        self.send_sender.send(messages).unwrap();
    }
}
