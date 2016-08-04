use specs;
use glutin;

use engine;

use glutin::Event::KeyboardInput;
use glutin::Event::MouseMoved;
use world::ExitFlag;
use engine::control::player::MoveEvent;
use engine::control::menu::{MenuEvent, MenuState};
use engine::control::camera::CameraMoveEvent;

/**
 * Send the events from the windowing system to event busses
 */
pub struct System {
  shift_held: bool,
}

impl System {
  pub fn new() -> System {
    System { shift_held: false }
  }
}

impl System {
  fn reposition_mouse(&self, window: &glutin::Window) {}
}

impl specs::System<engine::Delta> for System {
  fn run(&mut self, arg: specs::RunArg, _: engine::Delta) {
    use itertools::Itertools;

    let (mut glutin_events,
         window,
         menu_state,
         mut exit_flag,
         mut move_events,
         mut menu_events,
         mut camera_move_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<glutin::Event>>(),
       w.read_resource::<glutin::Window>(),
       w.read_resource::<MenuState>(),
       w.write_resource::<ExitFlag>(),
       w.write_resource::<Vec<MoveEvent>>(),
       w.write_resource::<Vec<MenuEvent>>(),
       w.write_resource::<Vec<CameraMoveEvent>>())
    });

    let mut router = EventRouter::new(&window,
                                      &menu_state,
                                      &mut self.shift_held,
                                      &mut exit_flag,
                                      &mut move_events,
                                      &mut menu_events,
                                      &mut camera_move_events);
    glutin_events.drain(..).foreach(|e| router.route_event(e));
  }
}

// TODO(acmcarther): Document
struct EventRouter<'a> {
  window: &'a glutin::Window,
  menu_state: &'a MenuState,
  shift_held: &'a mut bool,
  exit_flag: &'a mut ExitFlag,
  move_events: &'a mut Vec<MoveEvent>,
  menu_events: &'a mut Vec<MenuEvent>,
  camera_move_events: &'a mut Vec<CameraMoveEvent>,
}

impl<'a> EventRouter<'a> {
  pub fn new(window: &'a glutin::Window,
             menu_state: &'a MenuState,
             shift_held: &'a mut bool,
             exit_flag: &'a mut ExitFlag,
             move_events: &'a mut Vec<MoveEvent>,
             menu_events: &'a mut Vec<MenuEvent>,
             camera_move_events: &'a mut Vec<CameraMoveEvent>)
             -> EventRouter<'a> {
    EventRouter {
      window: window,
      menu_state: menu_state,
      shift_held: shift_held,
      exit_flag: exit_flag,
      move_events: move_events,
      menu_events: menu_events,
      camera_move_events: camera_move_events,
    }
  }

  pub fn route_event(&mut self, event: glutin::Event) {
    use glutin::VirtualKeyCode::{A, D, Escape, LShift, RShift, S, W};
    use glutin::ElementState;
    match (self.menu_state, event) {
      (menu_state, KeyboardInput(ElementState::Pressed, _, Some(Escape))) => {
        if *self.shift_held {
          *self.exit_flag = ExitFlag(true)
        } else {
          match menu_state {
            &MenuState::Open => self.menu_events.push(MenuEvent::Close),
            &MenuState::Closed => self.menu_events.push(MenuEvent::Open),
          };
        }
      },
      (_, KeyboardInput(ElementState::Pressed, _, Some(LShift))) => {
        *self.shift_held = true;
      },
      (_, KeyboardInput(ElementState::Released, _, Some(LShift))) => {
        *self.shift_held = false;
      },
      (_, KeyboardInput(ElementState::Pressed, _, Some(RShift))) => {
        *self.shift_held = true;
      },
      (_, KeyboardInput(ElementState::Released, _, Some(RShift))) => {
        *self.shift_held = false;
      },
      (&MenuState::Closed, KeyboardInput(ElementState::Pressed, _, Some(W))) => {
        self.move_events.push(MoveEvent::Forward)
      },
      (&MenuState::Closed, KeyboardInput(ElementState::Pressed, _, Some(A))) => {
        self.move_events.push(MoveEvent::Left)
      },
      (&MenuState::Closed, KeyboardInput(ElementState::Pressed, _, Some(S))) => {
        self.move_events.push(MoveEvent::Backward)
      },
      (&MenuState::Closed, KeyboardInput(ElementState::Pressed, _, Some(D))) => {
        self.move_events.push(MoveEvent::Right)
      },
      (&MenuState::Closed, MouseMoved(x, _)) => {
        // Move the mouse back to the middle of the window
        let (wx, wy) = self.window.get_position().unwrap();
        let (ox, oy) = self.window.get_outer_size().unwrap();
        let (middle_x, middle_y) = ((wx + ox as i32 / 2), (wy + oy as i32 / 2));
        self.window.set_cursor_position(middle_x, middle_y).unwrap();

        // Emit a camera event with a relative x move
        self.camera_move_events.push(CameraMoveEvent(x - middle_x))
      },
      _ => {},
    }
  }
}
