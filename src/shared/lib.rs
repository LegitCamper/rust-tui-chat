use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum ConnectionType {
    Echo(String),
    Message(String),
    Id(u32),
    IdRequest,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum PeerType {
    Server,
    Client(Option<u32>),
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ChatMessage {
    pub to: PeerType,
    pub from: PeerType,
    pub connect: ConnectionType,
}
