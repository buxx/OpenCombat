use crossbeam_channel::{unbounded, Receiver, Sender};
use ggez::GameResult;
use serde::{Deserialize, Serialize};

use std::thread;

use crate::config::Config;
use crate::message::*;

/// Network exchange logic
/// Important note : zmq PUB socket have a limited buffer size,
/// so we need to send messages by group instead one by one.
pub struct Network {
    config: Config,
    send_sender: Sender<Vec<Message>>,
    send_receiver: Receiver<Vec<Message>>,
    receive_sender: Sender<Vec<Message>>,
    receive_receiver: Receiver<Vec<Message>>,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Clone)]
struct Envelope {
    id: u64,
    messages: Vec<Message>,
}

impl Network {
    pub fn new(config: Config) -> GameResult<Self> {
        let (send_sender, send_receiver): (Sender<Vec<Message>>, Receiver<Vec<Message>>) =
            unbounded();
        let (receive_sender, receive_receiver): (Sender<Vec<Message>>, Receiver<Vec<Message>>) =
            unbounded();

        Ok(Self {
            config,
            send_sender,
            send_receiver,
            receive_sender,
            receive_receiver,
        })
    }

    pub fn init(&mut self) -> GameResult {
        match self.config.network_mode() {
            crate::NetWorkMode::Server => {
                self.start_rep()?;
                self.start_pub()?
            }
            crate::NetWorkMode::Client => {
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

    fn start_rep(&self) -> GameResult {
        let thread_receive_sender = self.receive_sender.clone();
        let server_rep_address = self.config.server_rep_address().clone();

        thread::spawn(move || {
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::REP).unwrap(); // TODO unwrap
            socket.bind(&server_rep_address).unwrap();

            let ok = bincode::serialize(&Message::Network(NetworkMessage::Acknowledge)).unwrap();
            loop {
                let messages_bytes = socket.recv_bytes(0).unwrap();
                let messages: Vec<Message> = bincode::deserialize(&messages_bytes).unwrap();
                // println!("Received REQ message: {:?}", message);
                socket.send(&ok, 0).unwrap();
                thread_receive_sender.send(messages).unwrap();
            }
        });

        Ok(())
    }

    fn start_req(&self) -> GameResult {
        let thread_send_receiver = self.send_receiver.clone();
        let server_rep_address = self.config.server_rep_address().clone();

        thread::spawn(move || {
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::REQ).unwrap(); // TODO unwrap
            socket.connect(&server_rep_address).unwrap();

            loop {
                let messages: Vec<Message> = thread_send_receiver.recv().unwrap();
                let messages_bytes = bincode::serialize(&messages).unwrap();
                socket.send(messages_bytes, 0).unwrap();
                let _response = socket.recv_bytes(0).unwrap();
                // FIXME : check this is OK or Error(error_str)
                // println!("Client recv : {:?}", response);
            }
        });

        Ok(())
    }

    fn start_pub(&self) -> GameResult {
        let thread_send_receiver = self.send_receiver.clone();
        let server_pub_address = self.config.server_pub_address().clone();

        thread::spawn(move || {
            let mut pub_counter: u64 = 0;
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::PUB).unwrap(); // TODO unwrap
            socket.bind(&server_pub_address).unwrap();

            loop {
                pub_counter += 1;
                let messages = thread_send_receiver.recv().unwrap();
                let envelope = Envelope {
                    id: pub_counter,
                    messages,
                };
                let messages_bytes = bincode::serialize(&envelope).unwrap();
                socket.send(&messages_bytes, 0).unwrap();
            }
        });

        Ok(())
    }

    fn start_sub(&self) -> GameResult {
        let thread_receive_sender = self.receive_sender.clone();
        let thread_send_sender = self.send_sender.clone();
        let server_pub_address = self.config.server_pub_address().clone();

        thread::spawn(move || {
            let mut last_counter: u64 = 0;
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::SUB).unwrap(); // TODO unwrap
            socket.connect(&server_pub_address).unwrap();

            // FIXME : subscribe with client ID and ALL (to receive all messages except global sync of other clients)
            socket.set_subscribe(b"").unwrap();

            loop {
                // println!("Client waiting message");
                let envelope_bytes = socket.recv_bytes(0).unwrap();
                let envelope: Envelope = bincode::deserialize(&envelope_bytes).unwrap();
                thread_receive_sender.send(envelope.messages).unwrap();

                if last_counter != 0 && last_counter + 1 != envelope.id {
                    println!("Network : message(s) lost, require global Sync");
                    thread_send_sender
                        .send(vec![Message::Network(NetworkMessage::RequireCompleteSync)])
                        .unwrap();
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
