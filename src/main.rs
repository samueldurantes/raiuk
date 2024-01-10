use std::{collections::HashMap, sync::Arc};
use raiuk::handler::{handle_connection, Channel};
use tokio::{
  net::TcpListener,
  sync::{broadcast, RwLock}
};

#[tokio::main]
async fn main() {
  let listener = TcpListener::bind("127.0.0.1:8000").await.unwrap();

  let channel: Channel = Arc::new(RwLock::new(HashMap::new()));
  let (sender, _) = broadcast::channel::<String>(32);

  loop {
    let (socket, addr) = listener.accept().await.unwrap();

    let channel = channel.clone();
    let sender = sender.clone();

    tokio::spawn(async move {
      handle_connection(socket, addr, channel, sender).await;
    });
  }
}
