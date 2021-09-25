mod twitch;

use std::{thread, time};
use crate::twitch::chat::IRCClient;

#[tokio::main]
pub async fn main() {
  println!("Hello, world!");
  let bits_closure = |amount_bits: u64| {
    println!("Bits {:1}", amount_bits);
  };

  let subs_closure = |amount_subs: u8| {
    println!("Subs {:1}", amount_subs);
  };

  let client = IRCClient::new("justintv392", None, bits_closure, subs_closure).await;


  //only works when IRCClient is supplied with an oauth token
  let can_send_msg = client.send_msg("yannismate", "test").await.is_ok();
  println!("Can send msg? {:?}", can_send_msg);


  //keepalive until http server is implemented
  loop {
    thread::sleep(time::Duration::from_secs(5));
  }

}
