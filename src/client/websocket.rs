use futures_util::{future, pin_mut, StreamExt};

use std::io::{self, stdout};
use std::{
    env,
    sync::{
        atomic::{
            AtomicU32,
            Ordering::{Acquire, Relaxed},
        },
        Arc,
    },
};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio_tungstenite::{connect_async, tungstenite::protocol::Message};

use shared::{send_message, ChatMessage, ConnectionType, PeerType};

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
            match message.connect {
                ConnectionType::Id(id) => {
                    client_id.compare_exchange(0, id, Relaxed, Relaxed).unwrap();
                    ();
                }
                ConnectionType::IdVerifyAck(bool) => {
                    if bool {
                        println!("Peer ID Verified")
                    } else {
                        panic!("Peer ID Incorrect")
                    }
                }
                ConnectionType::Echo(_) => println!("Echo: {:?}", message),
                ConnectionType::Message(msg) => {
                    if let PeerType::Client(client) = message.from {
                        if let Some(id) = client {
                            println!("Peer: {} says: {}", id, msg)
                        };
                    }
                }
                ConnectionType::PeerDisconnected => panic!("Peer Disconnected"),

                // thse are server only features
                ConnectionType::IdRequest => (),
                ConnectionType::IdVerify(_) => (),
            };
        })
    };

    pin_mut!(stdin_to_ws, ws_to_stdout);
    future::select(stdin_to_ws, ws_to_stdout).await;
}

pub fn get_client_id(
    tx: futures_channel::mpsc::UnboundedSender<Message>,
    client_id: Arc<AtomicU32>,
) {
    let chat_message = ChatMessage {
        to: PeerType::Server,
        from: PeerType::Client(None),
        connect: ConnectionType::IdRequest,
    };
    send_message(&tx, chat_message);
}

pub fn verify_peer_id(
    tx: futures_channel::mpsc::UnboundedSender<Message>,
    client_id: Arc<AtomicU32>,
    peer_id: u32,
) {
    let chat_message = ChatMessage {
        to: PeerType::Server,
        from: PeerType::Client(Some(client_id.load(Acquire))),
        connect: ConnectionType::IdVerify(peer_id),
    };
    send_message(&tx, chat_message);
}

pub async fn websocket(
    tx: futures_channel::mpsc::UnboundedSender<Message>,
    client_id: Arc<AtomicU32>,
) {
    get_client_id(tx.clone(), client_id.clone());

    // the following will be replaced by tui
    loop {
        if client_id.load(Acquire) != 0 {
            println!("Your Client ID is: {:?}", client_id.load(Acquire));
            break;
        }
    }

    // prompt for id to chat with
    println!("Peer ID: ");
    let mut peer_id = String::new();
    let stdin = io::stdin(); // We get `Stdin` here.
    stdin.read_line(&mut peer_id).unwrap();
    peer_id.pop(); // remove \n
    let peer_id: u32 = peer_id.parse().expect("Peer ID was not a number!");
    verify_peer_id(tx.clone(), client_id.clone(), peer_id);

    loop {
        let mut buffer = String::new();
        let stdin = io::stdin(); // We get `Stdin` here.
        stdin.read_line(&mut buffer).unwrap();
        buffer.pop(); // remove \n

        let chat_message = ChatMessage {
            to: PeerType::Client(Some(peer_id)),
            from: PeerType::Client(Some(client_id.load(Acquire))),
            connect: ConnectionType::Message(buffer),
        };
        send_message(&tx, chat_message);
    }
}
