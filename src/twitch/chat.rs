use actix::prelude::*;
use twitch_irc::{ClientConfig, TwitchIRCClient, SecureTCPTransport};
use twitch_irc::login::StaticLoginCredentials;
use expiring_map::ExpiringMap;
use std::time::Duration;
use twitch_irc::message::{ServerMessage, UserNoticeEvent};
use std::sync::{Arc, RwLock};

pub struct IrcClientActor {
  client: Option<TwitchIRCClient<SecureTCPTransport, StaticLoginCredentials>>,
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
      client: None,
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
#[rtype(result = "()")]
pub struct IrcConnect(pub String, pub Option<String>);


impl Handler<IrcConnect> for IrcClientActor {
  type Result = ();

  fn handle(&mut self, msg: IrcConnect, _ctx: &mut Self::Context) -> Self::Result {

    println!("{:?}", &msg);
    let config = ClientConfig::new_simple(StaticLoginCredentials::new(msg.0, msg.1));
    let (mut msg_rcvr, client) = TwitchIRCClient::new(config);

    //TODO: send connection state through second event

    self.client = Some(client);

    let cc_subs = self.cc_subscribers.clone();

    tokio::spawn(async move {
      let mut gift_bombs = ExpiringMap::<String, u64>::new(Duration::from_secs(30));

      //Move cc_subs in here
      let cc_subs = cc_subs;

      while let Some(msg) = msg_rcvr.recv().await {

        match msg {
          ServerMessage::Privmsg(priv_msg) => {
            if let Some(bits) = priv_msg.bits {
              broadcast_event(&cc_subs, IrcCcEvent{subs: 0, bits});
            }
          },
          ServerMessage::UserNotice(notice) => {
            match notice.event {
              UserNoticeEvent::SubOrResub { .. } => {
                //Single sub or resub
                broadcast_event(&cc_subs, IrcCcEvent{subs: 1, bits: 0});
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
                  broadcast_event(&cc_subs, IrcCcEvent{subs: 1, bits: 0});
                }
              },
              UserNoticeEvent::SubMysteryGift { mass_gift_count, .. } => {
                //Gift bomb, insert sender into map and pass on total amount
                gift_bombs.insert(notice.sender.id, mass_gift_count);
                broadcast_event(&cc_subs, IrcCcEvent{subs: mass_gift_count, bits: 0});
              },
              UserNoticeEvent::AnonSubMysteryGift { mass_gift_count, .. } => {
                //Anon gift bomb, insert anon id into map and pass on total amount
                gift_bombs.insert(String::from("274598607"), mass_gift_count);
                broadcast_event(&cc_subs, IrcCcEvent{subs: mass_gift_count, bits: 0});
              },
              _ => {}
            }
          },
          _ => {}
        }
      }

    });
    self.client.as_ref().unwrap().join(String::from("yannismate"));
  }
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