mod twitch;
mod web;

use actix_web::{App, HttpServer, middleware};
use crate::twitch::chat::IrcClientActor;
use actix::{Actor, Addr};

pub struct AppState {
  irc_client: Addr<IrcClientActor>,
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {

  let state = AppState {
    irc_client: IrcClientActor::default().start()
  };

  let state = actix_web::web::Data::new(state);

  HttpServer::new(move || {
    App::new()
      .wrap(middleware::Logger::default())
      .app_data(state.clone())
      .service(web::irc::irc_connect)
  }).bind("127.0.0.1:8080")?.run().await


}
