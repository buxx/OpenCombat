use oc_network::ToServer;
use oc_root::Client;

#[derive(Clone)]
pub enum Event<E: Client> {
    Connected(E),
    Disconnected(E),
    Message(E, ToServer),
}
