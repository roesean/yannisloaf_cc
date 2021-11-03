use actix::prelude::*;
use std::sync::{Arc, RwLock};
use websocket::{ClientBuilder, OwnedMessage};

const TWITCH_IRC_URL: &'static str = "ws://irc-ws.chat.twitch.tv:80";

pub struct IrcClientActor {
  is_logged_in: bool,
  cc_subscribers: Arc<RwLock<Vec<Recipient<IrcCcEvent>>>>
}

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct IrcCcEvent {
  subs: u64,
  bits: u64
}

impl Default for IrcClientActor {
  fn default() -> Self {
    IrcClientActor {
      is_logged_in: false,
      cc_subscribers: Arc::new(RwLock::new(Vec::<Recipient<IrcCcEvent>>::new()))
    }
  }
}

impl Actor for IrcClientActor {
  type Context = Context<Self>;
}

fn broadcast_event<A>(recipients: &Arc<RwLock<Vec<Recipient<A>>>>, event: A)
  where A: Message + Clone + Send, <A as actix::Message>::Result: Send {

  for rec in recipients.read().unwrap().iter() {
    rec.do_send(event.clone()).unwrap_or_else(|err| println!("Error sending event to subscriber: {:?}", err));
  }

}

/*
  IRC Connect
*/
#[derive(Message, Debug)]
#[rtype(result = "IrcConnectResult")]
pub struct IrcConnect(pub String, pub Option<String>);

#[derive(Debug)]
pub enum IrcConnectResult {
  Success,
  AuthFailed,
  ConnectionFailed
}


impl Handler<IrcConnect> for IrcClientActor {
  type Result = MessageResult<IrcConnect>;

  fn handle(&mut self, msg: IrcConnect, _ctx: &mut Self::Context) -> Self::Result {

    println!("{:?}", &msg);

    let client = ClientBuilder::new(TWITCH_IRC_URL)
        .unwrap()
        .connect_insecure();
    //Secure websockets cannot be split easily, may be possible with a custom tokio implementation

    if client.is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }
    let client = client.unwrap();

    let (mut receiver, mut sender) = client.split().unwrap();

    if msg.1.is_some() {
      self.is_logged_in = true;
      if sender.send_message(&to_msg(&format!("PASS {}", msg.1.unwrap()))).is_err() {
        return MessageResult(IrcConnectResult::ConnectionFailed);
      }
    }
    if sender.send_message(&to_msg(&format!("NICK {}", msg.0))).is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }

    let first_msg = receiver.recv_message();
    if first_msg.is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }
    match first_msg.unwrap() {
      OwnedMessage::Text(text) => {
        //Should be improved to be future-safe
        if text.contains("authentication failed") {
          return MessageResult(IrcConnectResult::AuthFailed);
        }
      },
      _ => {
        return MessageResult(IrcConnectResult::ConnectionFailed);
      }
    };
    std::thread::spawn(move || {
      while let Ok(msg) = receiver.recv_message() {

        println!("{:?}", msg);
      }
      println!("[WS] Close read loop");
    });

  MessageResult(IrcConnectResult::Success)
  }
}

#[inline]
fn to_msg(s: &String) -> websocket::Message {
  websocket::Message::text(s)
}


/*
  Add Listener
*/
#[derive(Message, Debug)]
#[rtype(result = "()")]
pub struct IrcCcSubscribe(pub Recipient<IrcCcEvent>);

impl Handler<IrcCcSubscribe> for IrcClientActor {
  type Result = ();

  fn handle(&mut self, msg: IrcCcSubscribe, _ctx: &mut Self::Context) -> Self::Result {
    self.cc_subscribers.write().unwrap().push(msg.0);
  }
}