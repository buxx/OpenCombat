use crossbeam_channel::{unbounded, Receiver, Sender};
use ggez::GameResult;

use std::thread;

use crate::config::Config;
use crate::message::*;

pub struct Network {
    config: Config,
    send_sender: Sender<Message>,
    send_receiver: Receiver<Message>,
    receive_sender: Sender<Message>,
    receive_receiver: Receiver<Message>,
}

impl Network {
    pub fn new(config: Config) -> GameResult<Self> {
        let (send_sender, send_receiver): (Sender<Message>, Receiver<Message>) = unbounded();
        let (receive_sender, receive_receiver): (Sender<Message>, Receiver<Message>) = unbounded();

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
        while let Ok(message) = self.receive_receiver.try_recv() {
            messages.push(message);
        }
        messages
    }

    fn start_rep(&self) -> GameResult {
        let thread_receive_sender = self.receive_sender.clone();
        let server_rep_address = self.config.server_rep_address().clone();

        thread::spawn(move || {
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::REP).unwrap(); // TODO unwrap
            socket
                .bind(&format!("tcp://{}", server_rep_address))
                .unwrap();

            let ok = bincode::serialize(&Message::Network(NetworkMessage::Acknowledge)).unwrap();
            loop {
                let message_bytes = socket.recv_bytes(0).unwrap();
                let message: Message = bincode::deserialize(&message_bytes).unwrap();
                println!("Received REQ message: {:?}", message);
                socket.send(&ok, 0).unwrap();
                thread_receive_sender.send(message).unwrap();
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
            socket
                .connect(&format!("tcp://{}", server_rep_address))
                .unwrap();

            loop {
                let message: Message = thread_send_receiver.recv().unwrap();
                let message_bytes = bincode::serialize(&message).unwrap();
                socket.send(message_bytes, 0).unwrap();
                let response = socket.recv_bytes(0).unwrap();
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
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::PUB).unwrap(); // TODO unwrap
            socket
                .bind(&format!("tcp://{}", server_pub_address))
                .unwrap();

            loop {
                let message = thread_send_receiver.recv().unwrap();
                println!("Broadcast message: {:?}", message);
                let message_bytes = bincode::serialize(&message).unwrap();
                socket.send(&message_bytes, 0).unwrap();
            }
        });

        Ok(())
    }

    fn start_sub(&self) -> GameResult {
        let thread_receive_sender = self.receive_sender.clone();
        let server_pub_address = self.config.server_pub_address().clone();

        thread::spawn(move || {
            let zmq_context = zmq::Context::new();
            let socket = zmq_context.socket(zmq::SUB).unwrap(); // TODO unwrap
            socket
                .connect(&format!("tcp://{}", server_pub_address))
                .unwrap();
            socket.set_subscribe(b"").unwrap();

            loop {
                println!("Client waiting message");
                let message_bytes = socket.recv_bytes(0).unwrap();
                let message: Message = bincode::deserialize(&message_bytes).unwrap();
                println!("Client received message: {:?}", message);
                thread_receive_sender.send(message).unwrap();
            }
        });

        Ok(())
    }

    pub fn send(&self, message: Message) {
        self.send_sender.send(message).unwrap();
    }
}
