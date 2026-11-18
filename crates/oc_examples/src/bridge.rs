use std::sync::mpsc::channel;

use oc_battle_gui::network::input::NetworkMessage;
use oc_network::{ToClient, ToServer};
use oc_world_server::bridge::Event;

pub fn bridge(
    server_rx: std::sync::mpsc::Receiver<((), ToClient)>,
    client_tx: std::sync::mpsc::Sender<Event<()>>,
) -> (
    std::sync::mpsc::Sender<ToServer>,
    std::sync::mpsc::Receiver<NetworkMessage>,
) {
    let (client_tx2, client_rx2) = channel::<ToServer>();
    std::thread::spawn(move || {
        let _ = client_tx.send(Event::Connected(()));
        while let Ok(message) = client_rx2.recv() {
            let _ = client_tx.send(Event::Message((), message));
        }
    });

    let (server_tx2, server_rx2) = channel::<NetworkMessage>();
    std::thread::spawn(move || {
        let _ = server_tx2.send(NetworkMessage::Connected);
        while let Ok((_, message)) = server_rx.recv() {
            let _ = server_tx2.send(NetworkMessage::Message(Box::new(message)));
        }
    });

    (client_tx2, server_rx2)
}
