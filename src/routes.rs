use ntex::web;
use ntex::util::Bytes;
use ntex::http::StatusCode;
use ntex::channel::mpsc::channel;
use futures::SinkExt;
use futures::channel::mpsc::unbounded;

use crate::event::*;
use crate::error::HttpError;

#[web::get("/subscribe")]
async fn subscribe_to_node(
  event_emitter: web::types::State<EventEmitter>,
) -> Result<web::HttpResponse, HttpError> {
  let mut event_emitter = event_emitter.as_ref();
  let (event_sender, event_receiver) = unbounded::<Bytes>();
  let (client_sender, client_receiver) = channel::<Result<Bytes, web::error::Error>>();
  if let Err(err) = event_emitter.send(Event::AddClient(event_sender)).await {
    return Err(HttpError {
      msg: format!("Unable to add client to the list {:?}", err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    });
  }
  EventClient::pipe_stream(event_receiver, client_sender);
  Ok(
    web::HttpResponse::Ok()
      .keep_alive()
      .content_type("nanocl/streaming-v1")
      .streaming(client_receiver),
  )
}

#[web::post("/publish")]
async fn publish_to_node(
  event_emitter: web::types::State<EventEmitter>,
) -> Result<web::HttpResponse, HttpError> {
  let mut event_emitter = event_emitter.as_ref();
  let message = EventMessage {
    name: String::from("Test name"),
    data: serde_json::json!({
      "hello": "test\ngg\n",
    }),
  };
  if let Err(err) = event_emitter.send(Event::Emit(message)).await {
    return Err(HttpError {
      msg: format!("Unable to send the message {:}", err),
      status: StatusCode::INTERNAL_SERVER_ERROR,
    });
  }
  Ok(web::HttpResponse::Ok().into())
}

// Register the routes
pub fn register(config: &mut web::ServiceConfig) {
  config.service(subscribe_to_node);
  config.service(publish_to_node);
}
