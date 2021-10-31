use actix::prelude::*;
use actix::dev::MessageResult;
use twitch_irc::{ClientConfig, TwitchIRCClient, SecureTCPTransport};
use twitch_irc::login::StaticLoginCredentials;
use expiring_map::ExpiringMap;
use std::time::Duration;
use tokio::sync::watch::{Sender, Receiver};
use twitch_irc::message::{ServerMessage, UserNoticeEvent};
use std::sync::{Arc, Mutex};

pub struct IrcClientActor {
  client: Option<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
  is_logged_in: bool,
  channel_tx: Arc<Mutex<Option<Sender<(u64, u64)>>>>,
  channel_rx: Arc<Mutex<Option<Receiver<(u64, u64)>>>>
}

impl Default for IrcClientActor {
  fn default() -> Self {
    IrcClientActor {
      client: None,
      is_logged_in: false,
      channel_tx: Arc::new(Mutex::new(None)),
      channel_rx: Arc::new(Mutex::new(None))
    }
  }
}

impl Actor for IrcClientActor {
  type Context = Context<Self>;
}


/*
  IRC Connect
*/
#[derive(Message, Debug)]
#[rtype(result = "IrcConnectResponse")]
pub struct IrcConnect(pub String, pub Option<String>);

#[derive(Debug)]
pub enum IrcConnectResponse {
  Success,
  ConnectionFailed,
  AuthFailed
}


impl Handler<IrcConnect> for IrcClientActor {
  type Result = MessageResult<IrcConnect>;

  fn handle(&mut self, msg: IrcConnect, _ctx: &mut Self::Context) -> Self::Result {

    println!("{:?}", &msg);
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(msg.0, msg.1));
    let (mut msg_rcvr, client) = TwitchIRCClient::new(config);

    //TODO: check connection state

    self.client = Some(client);

    let tx = self.channel_tx.clone();

    tokio::spawn(async move {
      let mut gift_bombs = ExpiringMap::<String, u64>::new(Duration::from_secs(30));

      //move tx into closure
      let tx = tx;

      while let Some(msg) = msg_rcvr.recv().await {
        if tx.lock().map_or(true, |tx|{tx.is_none()}) { continue; }

        match msg {
          ServerMessage::Privmsg(priv_msg) => {
            if let Some(bits) = priv_msg.bits {
              let locked_tx = tx.lock().unwrap();
              if locked_tx.is_some() {
               locked_tx.as_ref().unwrap().send((0, bits)).unwrap_or_else(|err| println!("{:?}", err));
              }
            }
          },
          ServerMessage::UserNotice(notice) => {
            match notice.event {
              UserNoticeEvent::SubOrResub { .. } => {
                //Single sub or resub
                let locked_tx = tx.lock().unwrap();
                if locked_tx.is_some() {
                  locked_tx.as_ref().unwrap().send((1, 0)).unwrap_or_else(|err| println!("{:?}", err));
                }
              },
              UserNoticeEvent::SubGift { is_sender_anonymous, .. } => {
                let sender_id = if is_sender_anonymous {String::from("274598607")} else {notice.sender.id};
                if *gift_bombs.get(&sender_id).unwrap_or(&0) > 0 {
                  //Sub gift part of bigger sub bomb, ignore
                  let gift_bomb_left = gift_bombs.get(&sender_id).unwrap_or(&0);
                  let gift_bomb_left = gift_bomb_left - 1;
                  gift_bombs.insert(sender_id, gift_bomb_left);
                } else {
                  //Single sub gift, not part of sub bomb
                  let locked_tx = tx.lock().unwrap();
                  if locked_tx.is_some() {
                    locked_tx.as_ref().unwrap().send((1, 0)).unwrap_or_else(|err| println!("{:?}", err));
                  }
                }
              },
              UserNoticeEvent::SubMysteryGift { mass_gift_count, .. } => {
                //Gift bomb, insert sender into map and pass on total amount
                gift_bombs.insert(notice.sender.id, mass_gift_count);
                let locked_tx = tx.lock().unwrap();
                if locked_tx.is_some() {
                  locked_tx.as_ref().unwrap().send((mass_gift_count, 0)).unwrap_or_else(|err| println!("{:?}", err));
                }
              },
              UserNoticeEvent::AnonSubMysteryGift { mass_gift_count, .. } => {
                //Anon gift bomb, insert anon id into map and pass on total amount
                gift_bombs.insert(String::from("274598607"), mass_gift_count);
                let locked_tx = tx.lock().unwrap();
                if locked_tx.is_some() {
                  locked_tx.as_ref().unwrap().send((mass_gift_count, 0)).unwrap_or_else(|err| println!("{:?}", err));
                }
              },
              _ => {}
            }
          },
          _ => {}
        }
      }

    });
    self.client.as_ref().unwrap().join(String::from("yannismate"));

    MessageResult(IrcConnectResponse::Success)
  }
}


/*
  Add Listener
*/
#[derive(Message, Debug)]
#[rtype(result = "IrcAddListenerResponse")]
pub struct IrcAddListener;

#[derive(Debug)]
pub struct IrcAddListenerResponse {
  pub receiver: Receiver<(u64, u64)>
}

impl Handler<IrcAddListener> for IrcClientActor {
  type Result = MessageResult<IrcAddListener>;

  fn handle(&mut self, msg: IrcAddListener, ctx: &mut Self::Context) -> Self::Result {
    let mut rx_lock = self.channel_rx.lock().unwrap();
    if rx_lock.is_none() {
      let mut tx_lock = self.channel_tx.lock().unwrap();
      let (tx, rx) = tokio::sync::watch::channel((0 as u64, 0 as u64));
      *tx_lock = Some(tx);
      *rx_lock = Some(rx);
    }
    let rx_copy = rx_lock.clone().unwrap().clone();
    MessageResult(IrcAddListenerResponse {receiver: rx_copy})
  }
}