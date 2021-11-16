use actix::{Actor, Context};
use winput::message_loop::Event;

pub struct InputActor;

impl Default for InputActor {
  fn default() -> Self {
    InputActor{}
  }
}

impl Actor for InputActor {
  type Context = Context<Self>;

  fn started(&mut self, ctx: &mut Self::Context) {
    std::thread::spawn(move || {
      let receiver = winput::message_loop::start().expect("Could not start Input loop!");
      println!("Input loop started");

      loop {
        match receiver.next_event() {
          Event::Keyboard { vk, scan_code, action } => {
            println!("Keyboard {:?} {:?} {:?}", vk, scan_code, action);
          },
          Event::MouseMoveRelative { x, y } => {
            /*
            Working code for inverted mouse:
            let (x_new, y_new) = winput::Mouse::position().unwrap();
            winput::Mouse::set_position(x_new-2*x, y_new-2*y);
             */
          },
          Event::MouseMoveAbsolute { .. } => {},
          Event::MouseButton { .. } => {},
          Event::MouseWheel { .. } => {}
        }
      }
    });
  }
}