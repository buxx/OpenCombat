use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use std::thread;

use crate::{
    channel::Channel,
    message::{InputMessage, OutputMessage},
};

use super::error::NetworkError;

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Client {
    req_address: String,
    sub_address: String,
    send_sender: Sender<Vec<InputMessage>>,
    send_receiver: Receiver<Vec<InputMessage>>,
    receive_sender: Sender<Vec<OutputMessage>>,
    receive_receiver: Receiver<Vec<OutputMessage>>,
    error_sender: Sender<NetworkError>,
    error_receiver: Receiver<NetworkError>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Envelope {
    id: u64,
    messages: Vec<OutputMessage>,
}

// TODO : When server/client is closing : end threads properly
impl Client {
    pub fn new(req_address: String, sub_address: String, channel: &Channel) -> Self {
        Self {
            req_address,
            sub_address,
            send_sender: channel.input_sender(),
            send_receiver: channel.input_receiver(),
            receive_sender: channel.output_sender(),
            receive_receiver: channel.output_receiver(),
            error_sender: channel.error_sender(),
            error_receiver: channel.error_receiver(),
        }
    }

    pub fn connect(&mut self) -> Result<(), NetworkError> {
        self.start_req()?;
        self.start_sub()?;
        Ok(())
    }

    /// Return received messages from remote :
    ///  - As server : messages from clients
    ///  - As client : messages from server
    pub fn incoming_messages(&self) -> Vec<OutputMessage> {
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

    fn start_req(&self) -> Result<(), NetworkError> {
        let thread_send_receiver = self.send_receiver.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_rep_address = self.req_address.clone();

        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::REQ)?;
        socket.connect(&server_rep_address)?;

        thread::spawn(move || {
            loop {
                // Wait messages to send
                let messages: Vec<InputMessage> = thread_send_receiver.recv().expect(&format!(
                    "Channel was closed when try to receive messages to send"
                ));

                // Encode messages to send
                let messages_bytes = match bincode::serialize(&messages) {
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

                // Send messages to server
                match socket.send(messages_bytes, 0) {
                    Err(error) => {
                        thread_error_sender
                            .send(NetworkError::SendError(format!(
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
                            .send(NetworkError::SendError(format!(
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

    fn start_sub(&self) -> Result<(), NetworkError> {
        let thread_receive_sender = self.receive_sender.clone();
        let thread_send_sender = self.send_sender.clone();
        let thread_error_sender = self.error_sender.clone();
        let server_pub_address = self.sub_address.clone();

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
                            .send(NetworkError::SendError(format!(
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
                            .send(NetworkError::ReceiveError(format!("Error while decoding received messages bytes : {}", error)))
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
                        .send(vec![InputMessage::RequireCompleteSync])
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

    pub fn send(&self, messages: Vec<InputMessage>) {
        self.send_sender.send(messages).unwrap();
    }
}
