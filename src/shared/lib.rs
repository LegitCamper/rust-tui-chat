use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum From {
    Peer(u32),
    Server,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ChatMessage {
    pub from_peer_id: From,
    pub to_peer_id: u32,
    pub message: String,
}
