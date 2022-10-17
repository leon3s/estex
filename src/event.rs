use std::collections::HashMap;
use std::sync::{Arc, Mutex, MutexGuard};

use ntex::rt;
use ntex::web;
use ntex::util::Bytes;
use ntex::channel::mpsc::Sender;
use futures::channel::mpsc;
use futures::channel::mpsc::UnboundedSender;
use futures::{stream, StreamExt, SinkExt};

type EventClients = HashMap<usize, EventClient>;
type EventSender = mpsc::UnboundedSender<Bytes>;
type ClientSocket = Sender<Result<Bytes, web::error::Error>>;
type EventReceiver = mpsc::UnboundedReceiver<Bytes>;

pub type EventEmitter = UnboundedSender<Event>;

#[derive(Debug, Clone)]
pub struct EventClient {
  id: usize,
  sender: EventSender,
}

impl EventClient {
  pub fn pipe_stream(mut source: EventReceiver, dest: ClientSocket) {
    rt::spawn(async move {
      while let Some(bytes) = source.next().await {
        if let Err(err) = dest.send(Ok::<_, web::error::Error>(bytes)) {
          eprintln!("Error while piping stream : {:} closing.", err);
          source.close();
          dest.close();
          break;
        }
      }
    });
  }
}

pub struct EventMessage {
  pub name: String,
  pub data: serde_json::Value,
}

pub enum Event {
  AddClient(EventSender),
  Emit(EventMessage)
}

#[derive(Clone)]
pub struct EventHandler {
  pub clients: Arc<Mutex<EventClients>>,
}

impl EventHandler {
  pub fn new() -> Self {
    Self {
      clients: Arc::new(Mutex::new(HashMap::new())),
    }
  }

  // Lock a mutex and return his mutex guard or none if the lock failed
  fn lock_mutex<T>(mutex: &'_ Arc<Mutex<T>>) -> Option<MutexGuard<'_, T>> {
    match mutex.lock() {
      Err(err) => {
        eprintln!("Unable to lock clients mutex {:#?}", err);
        None
      }
      Ok(guard) => Some(guard),
    }
  }

  // Serialize message
  fn serialize_message(message: &EventMessage) -> Option<String> {
    match serde_json::to_string(&message.data) {
      Err(err) => {
        eprintln!("Unable to serialize message error {:?}", err);
        None
      }
      Ok(s) => Some(format!("event: {}\ndata: {}\n", &message.name, &s)),
    }
  }

  fn remove_client(&self, id: usize) {
    if let Some(mut c) = Self::lock_mutex(&self.clients) {
      c.remove(&id);
    }
  }

  async fn emit_message(&self, message: &EventMessage) {
    let clients_ptr = match Self::lock_mutex(&self.clients) {
      None => HashMap::new(),
      Some(clients) => clients.clone(),
    };
    let mut stream = stream::iter(clients_ptr);
    while let Some((_id, mut client)) = stream.next().await {
      if let Some(data) = Self::serialize_message(message) {
        if let Err(_err) = client.sender.send(Bytes::from(data)).await {
          self.remove_client(client.id);
        }
      }
    }
  }

  fn add_client(&self, socket: &EventSender) -> Option<EventClient> {
    if let Some(mut lclients) = Self::lock_mutex(&self.clients) {
      let id = lclients.len() + 1;
      let client = EventClient {
        id,
        sender: socket.to_owned(),
      };
      lclients.insert(id, client.clone());
      return Some(client);
    }
    None
  }

  pub async fn handle_events(&mut self, event: Event) {
    match event {
      Event::AddClient(socket) => {
        self.add_client(&socket);
      }
      Event::Emit(message) => {
        self.emit_message(&message).await;
      }
    }
  }
}

pub async fn init() -> EventEmitter {
  let mut event_handler = EventHandler::new();
  let (tx, mut rx) = mpsc::unbounded::<Event>();
  // Handle events in a background thread
  rt::Arbiter::new().exec_fn(|| {
    rt::spawn(async move {
      // Start the event loop
      while  let Some(event) = rx.next().await {
        event_handler.handle_events(event).await;
      }
      // If the event loop stop, we stop the current thread
      rt::Arbiter::current().stop();
    });
  });

  tx
}
