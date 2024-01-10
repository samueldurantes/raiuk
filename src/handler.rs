use std::{net::SocketAddr, collections::HashMap, sync::Arc};
use tokio::{
  net::TcpStream,
  io::{AsyncWriteExt, BufReader, AsyncBufReadExt},
  sync::{broadcast::Sender, RwLock}
};

pub struct User {
  pub name: String,
  pub addr: SocketAddr,
}

impl User {
  pub fn new(name: &str, addr: SocketAddr) -> Self {
    User { name: name.to_owned(), addr }
  }
}

pub type Channel = Arc<RwLock<HashMap<SocketAddr, User>>>;

pub async fn handle_connection(
  mut socket: TcpStream, 
  addr: SocketAddr, 
  channel: Channel, 
  sender: Sender<String>,
) {
  println!("New connection: {}", addr);

  let mut receiver = sender.subscribe();

  loop {
    let (r, mut s) = socket.split();
    let mut reader = BufReader::new(r);
    let mut message = String::new();

    tokio::select! {
      message = receiver.recv() => {
        if let Ok(message) = message {
          s.write(message.as_bytes()).await.unwrap();
        }
      }

      result = reader.read_line(&mut message) => {
        let mut channel = channel.write().await;
        
        match result {
          Ok(_) => {
            match channel.get(&addr) {
              Some(user) => {
                let message = message.trim_end();

                sender.send(format!("[{}] {}\n", user.name, message)).unwrap();
              },
              None => {                
                let name = message.trim_end();

                channel.insert(addr, User::new(name, addr));

                sender.send(format!("{} join\n", name)).unwrap();
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
