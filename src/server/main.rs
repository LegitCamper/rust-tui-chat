use futures_channel::mpsc::{unbounded, UnboundedSender};
use futures_util::{future, pin_mut, stream::TryStreamExt, StreamExt};
use rand::random;
use std::{
    collections::HashMap,
    env,
    io::Error as IoError,
    net::SocketAddr,
    sync::{Arc, Mutex},
};
use tokio::net::{TcpListener, TcpStream};
use tokio_tungstenite::tungstenite::protocol::Message;

use shared::ChatMessage;

type Tx = UnboundedSender<Message>;
type PeerMap = Arc<Mutex<HashMap<u32, (Tx, SocketAddr)>>>;

async fn handle_connection(peer_map: PeerMap, raw_stream: TcpStream, addr: SocketAddr) {
    println!("Incoming TCP connection from: {}", addr);

    let ws_stream = tokio_tungstenite::accept_async(raw_stream)
        .await
        .expect("Error during the websocket handshake occurred");
    println!("WebSocket connection established: {}", addr);

    // Insert the write part of this peer to the peer map.
    let (tx, rx) = unbounded();
    peer_map.lock().unwrap().insert(random(), (tx, addr));

    let (outgoing, incoming) = ws_stream.split();

    let broadcast_incoming = incoming.try_for_each(|msg| {
        let chat_messege: ChatMessage = serde_json::from_str(msg.to_text().unwrap()).unwrap();
        let to_peer_id: u32 = chat_messege.to_peer_id;

        peer_map
            .lock()
            .unwrap()
            .get(&to_peer_id)
            .unwrap()
            .0
            .unbounded_send(msg.clone())
            .unwrap();

        future::ok(())
    });

    let receive_from_others = rx.map(Ok).forward(outgoing);

    pin_mut!(broadcast_incoming, receive_from_others);
    future::select(broadcast_incoming, receive_from_others).await;

    println!("{} disconnected", &addr);
    for (id, (_, ip)) in peer_map.lock().unwrap().iter() {
        if *ip == addr {
            peer_map.lock().unwrap().remove(id);
            break;
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), IoError> {
    let addr = env::args()
        .nth(1)
        .unwrap_or_else(|| "127.0.0.1:8080".to_string());

    let peer_map = Arc::new(Mutex::new(HashMap::new()));

    // Create the event loop and TCP listener we'll accept connections on.
    let try_socket = TcpListener::bind(&addr).await;
    let listener = try_socket.expect("Failed to bind");
    println!("Listening on: {}", addr);

    // Let's spawn the handling of each connection in a separate task.
    while let Ok((stream, addr)) = listener.accept().await {
        tokio::spawn(handle_connection(peer_map.clone(), stream, addr));
    }

    Ok(())
}
