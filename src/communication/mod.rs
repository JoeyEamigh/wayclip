use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};
use std::io::{self, prelude::*, BufReader};
use tracing::{info, warn};

use crate::{clipboard, menu};

pub struct SocketHandler {
  buffer: String,
  socket: SocketType,
}

pub enum SocketType {
  Server(LocalSocketListener),
  Client(LocalSocketStream),
}

impl SocketHandler {
  pub fn server() -> Self {
    let name = get_socket_name();

    let socket = match LocalSocketListener::bind(name) {
      Err(e) if e.kind() == io::ErrorKind::AddrInUse => panic!("Socket file occupied"),
      x => x.unwrap(),
    };

    info!("server socket opened at {}", name);

    Self {
      buffer: String::with_capacity(128),
      socket: SocketType::Server(socket),
    }
  }

  pub fn client() -> Self {
    let name = get_socket_name();

    let socket = match LocalSocketStream::connect(name) {
      Err(e) if e.kind() == io::ErrorKind::NotFound => panic!("Socket file not found"),
      x => x.unwrap(),
    };

    info!("client socket opened at {}", name);

    Self {
      buffer: String::with_capacity(128),
      socket: SocketType::Client(socket),
    }
  }

  pub fn listen(
    &mut self,
    clipboard: clipboard::WrappedClipboard,
    menu_message_sender: std::sync::mpsc::Sender<String>,
  ) {
    match &mut self.socket {
      SocketType::Server(listener) => {
        for conn in listener.incoming().filter_map(handle_error) {
          let mut conn = BufReader::new(conn);
          conn.read_line(&mut self.buffer).unwrap();
          info!("toggle from pid: {}", self.buffer);

          let clipboard = clipboard.clone();
          let menu_message_sender = menu_message_sender.clone();
          std::thread::spawn(move || {
            let mut menu = menu::init(clipboard).expect("failed to initialize a menu backend");
            let result = menu.show();

            let text = match result {
              Ok(Some(text)) => text,
              Ok(None) => return,
              Err(_) => return,
            };

            info!("selected: {:?}", text);
            menu_message_sender.send(text).unwrap();
          });

          self.buffer.clear();
        }
      }
      SocketType::Client(_) => panic!("Client cannot listen"),
    }
  }

  pub fn toggle(&mut self) {
    match &mut self.socket {
      SocketType::Client(conn) => {
        let pid = std::process::id();
        info!("my pid is {} and i am going to message the listener", pid);

        conn.write_all(format!("{}", pid).as_bytes()).unwrap();
      }
      SocketType::Server(_) => panic!("Server cannot toggle"),
    }
  }
}

fn get_socket_name() -> &'static str {
  use NameTypeSupport::*;
  match NameTypeSupport::query() {
    OnlyPaths => "/run/wayclip.sock",
    OnlyNamespaced | Both => "@wayclip.sock",
  }
}

fn handle_error(conn: io::Result<LocalSocketStream>) -> Option<LocalSocketStream> {
  match conn {
    Ok(c) => Some(c),
    Err(e) => {
      warn!("Incoming connection failed: {}", e);
      None
    }
  }
}
