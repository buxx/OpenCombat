use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crate::message::{InputMessage, OutputMessage};

use super::error::NetworkError;

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Client {
    req_address: String,
    sub_address: String,
    input_sender: Sender<Vec<InputMessage>>,
    input_receiver: Receiver<Vec<InputMessage>>,
    output_sender: Sender<Vec<OutputMessage>>,
    output_receiver: Receiver<Vec<OutputMessage>>,
    sync_required: Arc<AtomicBool>,
}

#[derive(Debug, Serialize, Deserialize, Clone)]
struct Envelope {
    id: u64,
    messages: Vec<OutputMessage>,
}

// TODO : When server/client is closing : end threads properly
impl Client {
    pub fn new(
        req_address: String,
        sub_address: String,
        input_sender: Sender<Vec<InputMessage>>,
        input_receiver: Receiver<Vec<InputMessage>>,
        output_sender: Sender<Vec<OutputMessage>>,
        output_receiver: Receiver<Vec<OutputMessage>>,
        sync_required: Arc<AtomicBool>,
    ) -> Self {
        Self {
            req_address,
            sub_address,
            input_sender,
            input_receiver,
            output_sender,
            output_receiver,
            sync_required,
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
        while let Ok(messages_) = self.output_receiver.try_recv() {
            messages.extend(messages_);
        }
        messages
    }

    fn start_req(&self) -> Result<(), NetworkError> {
        let thread_send_receiver = self.input_receiver.clone();
        let server_rep_address = self.req_address.clone();

        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::REQ)?;
        socket.connect(&server_rep_address)?;

        thread::Builder::new()
            .name("client_req".to_string())
            .spawn(move || {
                loop {
                    // Wait messages to send
                    let messages: Vec<InputMessage> = thread_send_receiver.recv().expect(&format!(
                        "Channel was closed when try to receive messages to send"
                    ));

                    // Encode messages to send
                    let messages_bytes = match bincode::serialize(&messages) {
                        Ok(messages_bytes) => messages_bytes,
                        Err(error) => {
                            println!("Error while encoding messages to send : {}", error);
                            continue;
                        }
                    };

                    // Send messages to server
                    match socket.send(messages_bytes, 0) {
                        Err(error) => {
                            println!("Error while sending messages : {}", error);
                            // Don't expect a response if send error
                            continue;
                        }
                        _ => {}
                    };

                    let _response = match socket.recv_bytes(0) {
                        Ok(response) => response,
                        Err(error) => {
                            println!("Error while receiving server REP : {}", error);
                            // Don't expect a response if send error
                            continue;
                        }
                    };

                    // Don't check the response content. The server ACK is only required here.
                }
            })
            .unwrap();

        Ok(())
    }

    fn start_sub(&self) -> Result<(), NetworkError> {
        let thread_receive_sender = self.output_sender.clone();
        let thread_input_sender = self.input_sender.clone();
        let server_pub_address = self.sub_address.clone();

        let mut last_counter: u64 = 0;
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::SUB)?;
        socket.connect(&server_pub_address)?;
        // TODO : subscribe with client ID and ALL (to receive all messages except global sync of other clients)
        socket.set_subscribe(b"")?;

        let sync_required_ = self.sync_required.clone();
        thread::Builder::new()
            .name("client_sub".to_string())
            .spawn(move || {
                loop {
                    // Receive server messages
                    let envelope_bytes = match socket.recv_bytes(0) {
                        Ok(envelope_bytes) => envelope_bytes,
                        Err(error) => {
                            println!("Error while receiving server messages : {}", error);
                            // Waiting again if receive error
                            continue;
                        }
                    };

                    // Decode received messages
                    let envelope: Envelope = match bincode::deserialize(&envelope_bytes) {
                        Ok(envelope) => envelope,
                        Err(error) => {
                            println!("Error while decoding received messages bytes : {}", error);
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
                        sync_required_.swap(true, Ordering::Relaxed);
                        thread_input_sender
                            .send(vec![InputMessage::RequireCompleteSync])
                            .expect(&format!(
                                "Channel was closed when try to send server sync requirement"
                            ));
                    }

                    // Update the last counter
                    last_counter = envelope.id;
                }
            })
            .unwrap();

        Ok(())
    }
}
