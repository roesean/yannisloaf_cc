mod twitch;

use actix_web::{App, HttpServer, middleware, Responder, HttpResponse, get};
use crate::twitch::chat::{IrcClientActor, IrcConnect};
use actix::{Actor, Addr};

pub struct AppState {
  irc_client: Addr<IrcClientActor>,
}

#[get("/")]
async fn hello(data: actix_web::web::Data<AppState>) -> impl Responder {
  let res = data.irc_client.send(IrcConnect(String::from("justinfan523"), None)).await;
  HttpResponse::Ok().body(format!("{:?}", res))
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
      .service(hello)
  }).bind("127.0.0.1:8080")?.run().await


}
