use specs;
use glutin;

use engine;
use std::convert::From;

use glutin::Event::KeyboardInput;
use glutin::Event::MouseMoved;
use engine::control::player::MoveEvent;
use engine::control::menu::{MenuEvent, MenuState};
use engine::control::camera::CameraMoveEvent;
use engine::control::console::ConsoleEvent;

/**
 * Send the events from the windowing system to event busses
 */
pub struct System;

impl System {
  pub fn new() -> System {
    System
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
         mut move_events,
         mut menu_events,
         mut camera_move_events,
         mut console_events) = arg.fetch(|w| {
      (w.write_resource::<Vec<glutin::Event>>(),
       w.read_resource::<glutin::Window>(),
       w.read_resource::<MenuState>(),
       w.write_resource::<Vec<MoveEvent>>(),
       w.write_resource::<Vec<MenuEvent>>(),
       w.write_resource::<Vec<CameraMoveEvent>>(),
       w.write_resource::<Vec<ConsoleEvent>>())
    });

    let mut router = EventRouter::new(&window,
                                      &menu_state,
                                      &mut move_events,
                                      &mut menu_events,
                                      &mut camera_move_events,
                                      &mut console_events);
    glutin_events.drain(..).foreach(|e| router.route_event(e));
  }
}

// TODO(acmcarther): Document
struct EventRouter<'a> {
  window: &'a glutin::Window,
  menu_state: &'a MenuState,
  move_events: &'a mut Vec<MoveEvent>,
  menu_events: &'a mut Vec<MenuEvent>,
  camera_move_events: &'a mut Vec<CameraMoveEvent>,
  console_events: &'a mut Vec<ConsoleEvent>,
}

impl<'a> EventRouter<'a> {
  pub fn new(window: &'a glutin::Window,
             menu_state: &'a MenuState,
             move_events: &'a mut Vec<MoveEvent>,
             menu_events: &'a mut Vec<MenuEvent>,
             camera_move_events: &'a mut Vec<CameraMoveEvent>,
             console_events: &'a mut Vec<ConsoleEvent>)
             -> EventRouter<'a> {
    EventRouter {
      window: window,
      menu_state: menu_state,
      move_events: move_events,
      menu_events: menu_events,
      camera_move_events: camera_move_events,
      console_events: console_events,
    }
  }

  pub fn route_event(&mut self, event: glutin::Event) {
    use glutin::VirtualKeyCode::{A, D, Escape, S, W};
    use glutin::ElementState;

    // Pipe events to console if menu is open
    if *self.menu_state == MenuState::Open {
      self.console_events.push(ConsoleEvent::from(event.clone()))
    }

    match (self.menu_state, event) {
      // Disabled, since menu was added. This will be moved into menu as a menu entry
      // KeyboardInput(_, _, Some(Escape)) => *self.exit_flag = ExitFlag(true),
      (&MenuState::Open, KeyboardInput(ElementState::Pressed, _, Some(Escape))) => {
        self.menu_events.push(MenuEvent::Close)
      },
      (&MenuState::Closed, KeyboardInput(ElementState::Pressed, _, Some(Escape))) => {
        self.menu_events.push(MenuEvent::Open)
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
