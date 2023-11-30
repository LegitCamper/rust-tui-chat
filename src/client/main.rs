mod tui;
mod websocket;

#[tokio::main]
async fn main() {
    websocket::connect_websocket().await
}
