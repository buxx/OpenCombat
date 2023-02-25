use crossbeam_channel::{Receiver, Sender};
use serde::{Deserialize, Serialize};

use std::{
    sync::{
        atomic::{AtomicBool, Ordering},
        Arc,
    },
    thread,
};

use crate::message::{network::NetworkMessage, InputMessage, Message, OutputMessage};

use super::error::NetworkError;

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Server {
    rep_address: String,
    pub_address: String,
    output_receiver: Receiver<Vec<OutputMessage>>,
    input_sender: Sender<Vec<InputMessage>>,
    stop_required: Arc<AtomicBool>,
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
    pub fn new(
        rep_address: String,
        pub_address: String,
        output_receiver: Receiver<Vec<OutputMessage>>,
        input_sender: Sender<Vec<InputMessage>>,
        stop_required: Arc<AtomicBool>,
    ) -> Self {
        Self {
            rep_address,
            pub_address,
            output_receiver,
            input_sender,
            stop_required,
        }
    }

    pub fn serve(&self) -> Result<(), NetworkError> {
        self.start_rep()?;
        self.start_pub()?;
        Ok(())
    }

    fn start_rep(&self) -> Result<(), NetworkError> {
        let thread_input_sender = self.input_sender.clone();
        let server_rep_address = self.rep_address.clone();

        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::REP)?;
        socket.bind(&server_rep_address)?;

        let ok = bincode::serialize(&Message::Network(NetworkMessage::Acknowledge)).unwrap();
        let stop_required_ = self.stop_required.clone();
        thread::Builder::new()
            .name("server_rep".to_string())
            .spawn(move || {
                loop {
                    // Receive client REQ messages bytes
                    let messages_bytes = match socket.recv_bytes(0) {
                        Ok(message_bytes) => message_bytes,
                        Err(error) => {
                            if stop_required_.load(Ordering::Relaxed) {
                                break;
                            }

                            println!("Error while receiving bytes : {}", error);
                            continue;
                        }
                    };

                    // Decode received bytes into collection of messages
                    let messages: Vec<InputMessage> = match bincode::deserialize(&messages_bytes) {
                        Ok(messages) => messages,
                        Err(error) => {
                            println!("Error while decoding received bytes : {}", error);
                            continue;
                        }
                    };

                    // Send client expected acknowledgement
                    socket.send(&ok, 0).unwrap_or_else(|e| {
                        println!("Error while sending acknowledgement : {}", e)
                    });

                    // Send through channel the decoded messages
                    thread_input_sender.send(messages).expect(&format!(
                        "Channel was closed when try to send received messages"
                    ));
                }

                println!("Server REP finished")
            })
            .unwrap();

        Ok(())
    }

    fn start_pub(&self) -> Result<(), NetworkError> {
        let thread_output_receiver = self.output_receiver.clone();
        let server_pub_address = self.pub_address.clone();

        let mut pub_counter: u64 = 0;
        let zmq_context = zmq::Context::new();
        let socket = zmq_context.socket(zmq::PUB)?;
        socket.bind(&server_pub_address)?;

        let stop_required_ = self.stop_required.clone();
        thread::Builder::new()
            .name("server_pub".to_string())
            .spawn(move || {
                loop {
                    // Increment counter to permit client to know if some messages have been lost
                    pub_counter += 1;

                    // Retrieve messages to sent to clients
                    let messages: Vec<OutputMessage> = match thread_output_receiver.recv() {
                        Ok(messages) => messages,
                        Err(error) => {
                            if !stop_required_.load(Ordering::Relaxed) {
                                println!(
                                    "Channel was closed when try to receive messages to send : {}",
                                    error
                                )
                            }
                            break;
                        }
                    };

                    // Prepare the data to send to clients
                    let envelope = Envelope::new(pub_counter, messages);
                    let messages_bytes = match bincode::serialize(&envelope) {
                        Ok(messages_bytes) => messages_bytes,
                        Err(error) => {
                            println!("Error while encoding messages to send : {}", error);
                            continue;
                        }
                    };

                    // Finally send messages to clients
                    match socket.send(&messages_bytes, 0) {
                        Err(error) => {
                            println!("Error while sending messages : {}", error);
                        }
                        _ => {}
                    };
                }
                println!("Server PUB finished");
            })
            .unwrap();

        Ok(())
    }
}
