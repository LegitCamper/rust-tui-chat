use serde::{Deserialize, Serialize};
use std::sync::{atomic::AtomicU32, Arc};
use tokio_tungstenite::tungstenite::protocol::Message;

#[derive(Deserialize, Serialize, Debug)]
pub enum ConnectionType {
    Echo(String),
    Message(String),
    Id(u32),
    IdRequest,
    IdVerify(u32),
    IdVerifyAck(bool),
    PeerDisconnected,
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

pub fn send_message(
    tx: &futures_channel::mpsc::UnboundedSender<Message>,
    chat_message: ChatMessage,
) {
    let message = Message::Text(serde_json::to_string(&chat_message).unwrap());
    tx.unbounded_send(message).unwrap();
}
