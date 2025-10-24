use interprocess::local_socket::{LocalSocketListener, LocalSocketStream, NameTypeSupport};
use std::io::{self, prelude::*, BufReader};
use tracing::{debug, warn};

use crate::{clipboard, menu};

pub struct SocketHandler {
  buffer: String,
  socket: SocketType,
}

pub enum SocketType {
  Server(LocalSocketListener),
  Client(LocalSocketStream),
}

pub type MPSCMessage = (String, usize);

impl SocketHandler {
  pub fn server() -> Self {
    let name = get_socket_name();

    let socket = match LocalSocketListener::bind(name) {
      Err(e) if e.kind() == io::ErrorKind::AddrInUse => {
        eprintln!("socket already in use, please check that wayclip is not already running");
        std::process::exit(1);
      }
      x => x.unwrap(),
    };

    debug!("server socket opened at {}", name);

    Self {
      buffer: String::with_capacity(128),
      socket: SocketType::Server(socket),
    }
  }

  pub fn client() -> Self {
    let name = get_socket_name();

    let socket = match LocalSocketStream::connect(name) {
      Err(e) if e.kind() == io::ErrorKind::NotFound => {
        eprintln!("wayclip server is not running, please start it first");
        std::process::exit(1);
      }
      x => x.unwrap(),
    };

    debug!("client socket opened at {}", name);

    Self {
      buffer: String::with_capacity(128),
      socket: SocketType::Client(socket),
    }
  }

  pub fn listen(
    &mut self,
    clipboard: clipboard::WrappedClipboard,
    menu_message_sender: std::sync::mpsc::Sender<MPSCMessage>,
  ) {
    let menu = menu::init(clipboard).expect("failed to initialize a menu backend");

    match &mut self.socket {
      SocketType::Server(listener) => {
        for conn in listener.incoming().filter_map(handle_error) {
          let mut conn = BufReader::new(conn);
          conn.read_line(&mut self.buffer).unwrap();
          debug!("server got toggle from client pid: {}", self.buffer);

          let result = &menu.show();

          let data = match result {
            Ok(Some(data)) => data,
            Ok(None) => continue,
            Err(_) => continue,
          };

          debug!("selected: \"{:?}\" from menu of index \"{:?}\"", data.0, data.1);
          menu_message_sender.send(data.clone()).unwrap();

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
        debug!("my (client) pid is {} and i am going to message the server", pid);

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
