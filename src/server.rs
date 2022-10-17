use ntex::web;

use crate::event::EventEmitter;
use crate::routes::register;

/// Start the webserver and bind it to given ip_address and port
/// Also pass the event emiter as state
pub async fn start(
  ip_address: &str,
  port: &i32,
  event_emitter: EventEmitter,
) -> std::io::Result<()> {
  let mut server = web::HttpServer::new(move || {
    web::App::new().state(event_emitter.clone()).configure(register)
  });
  let host = format!("{}:{}", &ip_address, &port);
  server = server.bind(&host)?;
  println!("server listening on http://{}", &host);
  server.run().await?;
  Ok(())
}
