use std::collections::HashMap;
use std::error::Error;
use std::fmt::{Debug, Display, Formatter};
use websocket::Message;

//Sending
pub enum IrcCommand {
  Nick(NICKCMD),
  Pass(PASSCMD),
  Join(JOINCMD),
  Part(PARTCMD),
  Privmsg(PRIVMSGCMD),
  CapReq(CAPREQCMD),
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

pub type NICKCMD = String;
pub type PASSCMD = String;
pub type JOINCMD = String;
pub type PARTCMD = String;
pub type CAPREQCMD = IrcCapabilities;

pub struct PRIVMSGCMD {
  channel: String,
  msg: String
}

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
#[derive(Debug)]
pub enum IrcMessage {
  Notice(IrcMessageContent),
  Usernotice(IrcMessageContent),
  Privmsg(IrcMessageContent),
  Ping,
  Other
}

impl IrcMessage {
  pub fn parse(str: String) -> Result<IrcMessage, Box<dyn Error>> {
    println!("Parsing {:?}", &str);
    if str.is_empty() {}
    let mut tags: HashMap<String, String> = HashMap::new();
    let mut str = str.chars().into_iter().peekable();

    if str.peek().is_some() && str.peek().unwrap().eq(&'@') {
      let tag_str : String = str.by_ref().take_while(|a| {a.ne(&' ')}).collect();
      let tag_str : String = tag_str.chars().skip(1).take(tag_str.len() - 1).collect();

      for tag in tag_str.split(';') {
        let mut spl = tag.split('=');
        let name = spl.next().unwrap_or("");
        let val = spl.next().unwrap_or("");
        tags.insert(String::from(name), String::from(val));
      }
    }


    let user : String = str.by_ref().skip(1).take_while(|c| {c.ne(&' ')}).collect();
    let user = if user.contains('!') {Some(String::from(user.split('!').next().unwrap()))} else {None};

    let msg_type : String = str.by_ref().take_while(|a| {a.ne(&' ')}).collect();

    let channel : String = str.by_ref().skip(1).take_while(|a| {a.ne(&' ')}).collect();
    let channel = if channel.len() > 0 {Some(channel)} else {None};

    let msg : String = str.skip_while(|a| {a.ne(&':')}).skip(1).collect();

    let content = IrcMessageContent{
      tags,
      user,
      channel,
      msg
    };

    match &*msg_type {
      "PRIVMSG" => Ok(IrcMessage::Privmsg(content)),
      "NOTICE" => Ok(IrcMessage::Notice(content)),
      "USERNOTICE" => Ok(IrcMessage::Usernotice(content)),
      "PING" => Ok(IrcMessage::Ping),
      _ => Ok(IrcMessage::Other)
    }

  }
}

#[derive(Debug)]
pub struct IrcMessageContent {
  pub tags: HashMap<String, String>,
  pub user: Option<String>,
  pub channel: Option<String>,
  pub msg: String
}

pub struct IrcParseError;

impl Debug for IrcParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("Error parsing IRC Message")
  }
}

impl Display for IrcParseError {
  fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
    f.write_str("Error parsing IRC Message")
  }
}

impl Error for IrcParseError {}