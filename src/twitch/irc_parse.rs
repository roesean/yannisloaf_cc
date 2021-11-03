use std::fmt::{Display, Formatter};
use websocket::Message;


//Sending
pub enum IrcCommand {
  Nick(NICK),
  Pass(PASS),
  Join(JOIN),
  Part(PART),
  Privmsg(PRIVMSG),
  CapReq(CAPREQ),
  Raw(String)
}


impl IrcCommand {
  pub fn to_message(&self) -> Message {
    let str = match self {
      IrcCommand::Nick(name) => format!("NICK {}", name.to_lowercase()),
      IrcCommand::Pass(token) => {
        if token.starts_with("oauth:") {
          format!("PASS {}", token)
        } else {
          format!("PASS oauth:{}", token)
        }
      },
      IrcCommand::Join(channel) => format!("JOIN #{}", channel),
      IrcCommand::Part(channel) => format!("PART #{}", channel),
      IrcCommand::Privmsg(privmsg) => format!("PRIVMSG #{} :{}", privmsg.channel, privmsg.msg),
      IrcCommand::CapReq(capability) => format!("CAP REQ :twitch.tv/{}", capability.to_string()),
      IrcCommand::Raw(raw) => raw.clone(),
    };
    Message::text(str)
  }
}

pub type NICK = String;
pub type PASS = String;
pub type JOIN = String;
pub type PART = String;
pub struct PRIVMSG {
  channel: String,
  msg: String
}
pub type CAPREQ = IrcCapabilities;

pub enum IrcCapabilities {
  Membership,
  Tags,
  Commands
}
impl Display for IrcCapabilities {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    match self {
      IrcCapabilities::Membership => write!(f, "membership"),
      IrcCapabilities::Tags => write!(f, "tags"),
      IrcCapabilities::Commands => write!(f, "commands")
    }
  }
}

//Receiving
pub enum IrcMessage {
  Notice(NOTICE),
  Usernotice(USERNOTICE),
  Privmsg(PRIVMSG),
  Other
}