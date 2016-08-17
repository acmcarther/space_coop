use cucumber::CucumberRegistrar;

use support::{ClientWorld, window};
use std::str::FromStr;
use pubsub::PubSubStore;
use glutin::{ElementState, Event, VirtualKeyCode, Window};
use mouse_lock::RelativeMouseMovementEvent;

pub fn register_steps(c: &mut CucumberRegistrar<ClientWorld>) {

  When!(c,
        "^the \'(.*)\' key is pressed$",
        |_, world: &mut ClientWorld, (ch,): (String,)| {
    let code = window::str_to_virtual_key_code(&ch).unwrap();
    let mut key_pub = world.planner.mut_world().fetch_publisher::<Event>();

    key_pub.push(Event::KeyboardInput(ElementState::Pressed, 0, Some(code.clone())));
    key_pub.push(Event::KeyboardInput(ElementState::Released, 0, Some(code)));
  });

  When!(c,
        "^the mouse is moved left$",
        |_, world: &mut ClientWorld, _| {
          let mut relative_mouse_movement_pub =
            world.planner.mut_world().fetch_publisher::<RelativeMouseMovementEvent>();
          relative_mouse_movement_pub.push(RelativeMouseMovementEvent { x: -5, y: 0 });
        });

  When!(c,
        "^the mouse is moved right$",
        |_, world: &mut ClientWorld, _| {
          let mut relative_mouse_movement_pub =
            world.planner.mut_world().fetch_publisher::<RelativeMouseMovementEvent>();
          relative_mouse_movement_pub.push(RelativeMouseMovementEvent { x: 5, y: 0 });
        });
}
