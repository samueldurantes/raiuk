use std::{net::SocketAddr, collections::HashMap, sync::Arc};
use tokio::{
  net::TcpStream,
  io::{AsyncWriteExt, BufReader, AsyncBufReadExt},
  sync::{broadcast::Sender, RwLock}
};

#[derive(Debug, Clone)]
pub struct User {
  pub name: String,
  pub addr: SocketAddr,
}

impl User {
  pub fn new(name: &str, addr: SocketAddr) -> Self {
    User { name: name.to_owned(), addr }
  }
}

#[derive(Debug, Clone)]
pub struct Message {
  pub content: String,
  pub sender: User,
}

impl Message {
  pub fn new(content: &str, sender: User) -> Self {
    Message { content: content.to_owned(), sender }
  }
}

pub type Channel = Arc<RwLock<HashMap<SocketAddr, User>>>;

pub async fn handle_connection(
  mut socket: TcpStream,
  addr: SocketAddr,
  channel: Channel,
  sender: Sender<Message>,
) {
  println!("New connection: {}", addr);

  let mut receiver = sender.subscribe();

  loop {
    let (r, mut s) = socket.split();
    let mut reader = BufReader::new(r);
    let mut message = String::new();

    tokio::select! {
      message = receiver.recv() => {
        let message = message.unwrap();

        if message.sender.addr == addr {
          continue;
        }

        s.write(message.content.as_bytes()).await.unwrap();
      }

      result = reader.read_line(&mut message) => {
        let mut channel = channel.write().await;

        match result {
          Ok(_) => {
            match channel.get(&addr) {
              Some(user) => {
                let message = Message::new(
                  &format!("[{}] {}\n", user.name, message.trim_end()),
                  user.to_owned(),
                );

                sender.send(message).unwrap();
              },
              None => {
                let name = message.trim_end();
                let new_user = User::new(name, addr);

                channel.insert(addr, new_user.clone());

                let message = Message::new(
                  &format!("{} join\n", new_user.name),
                  new_user,
                );

                sender.send(message).unwrap();
              },
            }
          },
          Err(_) => {
            channel.remove(&addr);

            println!("Disconnect: {}", addr);

            break;
          },
        }
      }
    }
  }
}
