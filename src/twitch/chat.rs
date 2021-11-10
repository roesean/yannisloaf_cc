use std::net::TcpStream;
use actix::prelude::*;
use std::sync::{Arc, Mutex, RwLock};
use websocket::{ClientBuilder, OwnedMessage};
use websocket::client::sync::Writer;
use crate::twitch::irc_parse::{IrcCapabilities, IrcCommand, IrcMessage};

const TWITCH_IRC_URL: &'static str = "ws://irc-ws.chat.twitch.tv:80";

pub struct IrcClientActor {
  is_logged_in: bool,
  cc_subscribers: Arc<RwLock<Vec<Recipient<IrcCcEvent>>>>,
  sender: Arc<Mutex<Option<Writer<TcpStream>>>>
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
      cc_subscribers: Arc::new(RwLock::new(Vec::<Recipient<IrcCcEvent>>::new())),
      sender: Arc::new(Mutex::new(None))
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
    {
      let sender_lock = self.sender.lock().unwrap();
      if sender_lock.is_some() {
        sender_lock.as_ref().unwrap().shutdown_all().unwrap_or_else(|err| println!("Could not shutdown old websocket: {:?}", err))
      }
    }

    println!("{:?}", &msg);

    let client = ClientBuilder::new(TWITCH_IRC_URL)
        .unwrap()
        .connect_insecure();
    //Secure websockets cannot be split easily, may be possible with a custom tokio implementation

    if client.is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }
    let client = client.unwrap();

    let (mut receiver, raw_sender) = client.split().unwrap();

    //Update sender
    let mut sender = self.sender.lock().unwrap();
    *sender = Some(raw_sender);

    let sender_handle = self.sender.clone();

    let sender = sender.as_mut().unwrap();

    if msg.1.is_some() {
      self.is_logged_in = true;
      if sender.send_message(&IrcCommand::Pass(msg.1.unwrap()).to_message()).is_err() {
        return MessageResult(IrcConnectResult::ConnectionFailed);
      }
    }
    if sender.send_message(&IrcCommand::Nick(msg.0).to_message()).is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }

    let first_msg = receiver.recv_message();
    if first_msg.is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }
    match first_msg.unwrap() {
      OwnedMessage::Text(text) => {
        if let Ok(msg) = IrcMessage::parse(text) {
          match &msg {
            IrcMessage::Notice(notice) => {
              if notice.msg.starts_with("Login authentication") {
                return MessageResult(IrcConnectResult::AuthFailed);
              }
            },
            _ => {}
          };
        } else {
          return MessageResult(IrcConnectResult::ConnectionFailed);
        }
      },
      _ => {
        return MessageResult(IrcConnectResult::ConnectionFailed);
      }
    };

    std::thread::spawn(move || {
      let sender_handle = sender_handle;
      while let Ok(msg) = receiver.recv_message() {
        match msg {
          OwnedMessage::Text(text) => {
            for line in text.lines() {
              if line.starts_with("PING") {
                sender_handle.lock().unwrap().as_mut().unwrap().send_message(&to_msg(&String::from("PONG :tmi.twitch.tv"))).unwrap();
                continue;
              }
              let msg = IrcMessage::parse(String::from(line)).unwrap();
              println!("[IRC] {:?}", msg);
            }
          },
          _ => {}
        }
      }
      println!("[WS] Close read loop");
    });

    if sender.send_message(&IrcCommand::CapReq(IrcCapabilities::Commands).to_message()).is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }

    if sender.send_message(&IrcCommand::CapReq(IrcCapabilities::Tags).to_message()).is_err() {
      return MessageResult(IrcConnectResult::ConnectionFailed);
    }

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