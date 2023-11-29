use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize, Debug)]
pub enum ServerRequest {
    Echo,
    PeerId,
}
#[derive(Deserialize, Serialize, Debug)]
pub enum ConnectionType {
    Peer(u32),
    ToServer(ServerRequest),
    FromServer(u32),
    Server,
}
#[derive(Deserialize, Serialize, Debug)]
pub struct ChatMessage {
    pub from_peer_id: ConnectionType,
    pub to_peer_id: ConnectionType,
    pub message: Option<String>,
}
