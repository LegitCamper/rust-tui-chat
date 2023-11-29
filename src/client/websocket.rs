use futures_util::{future, pin_mut, StreamExt};

use std::io::{self, stdout};
use std::{
    env,
    sync::{
        atomic::{AtomicU32, Ordering::Relaxed},
        Arc,
    },
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use shared::{ChatMessage, ConnectionType, PeerType};

pub async fn connect_websocket() {
    let connect_addr = env::args()
        .nth(1)
        .unwrap_or_else(|| panic!("this program requires at least one argument"));

    let url = url::Url::parse(&connect_addr).unwrap();

    let (stdin_tx, stdin_rx) = futures_channel::mpsc::unbounded();

    // requests a client id from the server
    let client_id: Arc<AtomicU32> = Arc::new(0.into());
    tokio::spawn(websocket(stdin_tx, client_id.clone()));

    let (ws_stream, _) = connect_async(url).await.expect("Failed to connect");
    println!("WebSocket handshake has been successfully completed");

    let (write, read) = ws_stream.split();

    let stdin_to_ws = stdin_rx.map(Ok).forward(write);
    let ws_to_stdout = {
        read.for_each(|message| async {
            let client_id = client_id.clone();
            let message: ChatMessage =
                serde_json::from_str(&message.unwrap().into_text().unwrap()).unwrap();
            if let ConnectionType::Id(id) = message.connect {
                client_id.compare_exchange(0, id, Relaxed, Relaxed).unwrap();
            }
            println!("{:?}", message);
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

pub async fn websocket(
    tx: futures_channel::mpsc::UnboundedSender<Message>,
    client_id: Arc<AtomicU32>,
) {
    // gets client id from server
    let chat_message = ChatMessage {
        to: PeerType::Server,
        from: PeerType::Client(None),
        connect: ConnectionType::IdRequest,
    };
    let message = Message::Text(serde_json::to_string(&chat_message).unwrap());
    tx.unbounded_send(message).unwrap();
    println!("Your Client ID is: {:?}", client_id);

    loop {
        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).unwrap();

        let chat_message = ChatMessage {
            to: PeerType::Server,
            from: PeerType::Client(Some(client_id.load(Relaxed))),
            connect: ConnectionType::Echo(buffer),
        };
        let message = Message::Text(serde_json::to_string(&chat_message).unwrap());
        tx.unbounded_send(message).unwrap();
    }
}
