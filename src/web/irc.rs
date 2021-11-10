use actix_web::web::Json;
use serde::Deserialize;
use actix_web::{Responder, HttpResponse, post};
use crate::AppState;
use crate::twitch::chat::IrcConnect;

#[derive(Deserialize)]
pub struct IrcConnectData {
  username: String,
  #[serde(default)]
  token: Option<String>
}
#[post("/api/irc/connect")]
pub async fn irc_connect(irc_connect_data: Json<IrcConnectData>, data: actix_web::web::Data<AppState>) -> impl Responder {
  let res = data.irc_client.send(IrcConnect(irc_connect_data.username.clone(), irc_connect_data.token.clone())).await;
  HttpResponse::Ok().body(format!("{:?}", res))
}