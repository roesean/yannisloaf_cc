use twitch_irc::{ClientConfig, TwitchIRCClient, SecureTCPTransport};
use twitch_irc::login::StaticLoginCredentials;
use twitch_irc::message::ServerMessage;
use twitch_irc::message::UserNoticeEvent;
use std::convert::TryFrom;
use std::time::Duration;
use expiring_map::ExpiringMap;
use std::error::Error;

pub struct IRCClient {
  client: TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>,
  is_logged_in: bool
}

impl IRCClient {
  pub async fn new<F: Fn(u64) + 'static + Sync + Send, G: Fn(u8) + 'static + Sync + Send>(username: &str, token: Option<&str>, bits_closure: F, subs_closure: G) -> IRCClient {

    let config = match token {
      Some(token) => ClientConfig::new_simple(StaticLoginCredentials::new(username.to_string(), Some(token.to_string()))),
      None => ClientConfig::new_simple(StaticLoginCredentials::new("justinfan5133".to_string(), None))
    };


    let (mut incoming_messages, client) =
      TwitchIRCClient::<SecureTCPTransport, StaticLoginCredentials>::new(config);

    tokio::spawn(async move {

      let mut gift_bombs = ExpiringMap::new(Duration::from_secs(30));

      while let Some(message) = incoming_messages.recv().await {
        match message {
          ServerMessage::Privmsg(priv_msg) => {
            match priv_msg.bits {
              Some(amount) => {
                bits_closure(amount);
              },
              None => {}
            }
          },
          ServerMessage::UserNotice(notice) => {
            match notice.event {
              UserNoticeEvent::SubMysteryGift {mass_gift_count, ..} => {
                //Multiple gifted subs
                gift_bombs.insert(notice.sender.id, mass_gift_count);
                subs_closure(u8::try_from(mass_gift_count).unwrap_or(0));
              },
              UserNoticeEvent::AnonSubMysteryGift {mass_gift_count, ..}=> {
                //Multiple gifted subs
                gift_bombs.insert("274598607".to_string(), mass_gift_count);
                subs_closure(u8::try_from(mass_gift_count).unwrap_or(0));
              },
              UserNoticeEvent::SubGift {is_sender_anonymous, ..} => {
                let sender_id = match is_sender_anonymous {
                  true => "274598607".to_string(),
                  false => notice.sender.id
                };
                //Single gifted sub, can be part of a sub-bomb
                if *gift_bombs.get(&sender_id).unwrap_or(&0) > 0 {
                  //Part of the sub bomb
                  let gift_bomb_left = gift_bombs.get(&sender_id).unwrap_or(&0);
                  let gift_bomb_left = gift_bomb_left - 1;
                  gift_bombs.insert(sender_id, gift_bomb_left);
                } else {
                  //Single gift
                  subs_closure(1);
                }
              },
              UserNoticeEvent::SubOrResub {..} => {
                //Normal sub or resub
                subs_closure(1);
              },
              _ => {}
            };

          },
          _ => {}
        }
      }
    });

    IRCClient {
      client,
      is_logged_in: token.is_some()
    }

  }

  pub async fn join_channel(&self, channel: &str) {
    self.client.join(channel.to_string());
  }

  pub async fn part_channel(&self, channel: &str) {
    self.client.part(channel.to_string());
  }

  pub async fn send_msg(&self, channel: &str, msg: &str) -> Result<(), Box<dyn Error>> {
    if !self.is_logged_in {
      Err("IRC Client is not logged in.")?
    }
    match self.client.say(channel.to_string(), msg.to_string()).await {
      Ok(()) => Result::Ok(()),
      Err(reason) => Err(&*reason.to_string())?
    }
  }

}