pub use self::client::{start};

mod client {
  use params;

  use app_net::ClientNet;
  use events::{
    ClientEvent,
    ServerEvent
  };
  use state::ClientState;

  use std::thread;
  use std::io::stdin;
  use std::sync::mpsc::channel;

  use itertools::Itertools;
  use time::SteadyTime;

  use glutin;
  use gfx_window_glutin;
  use gfx;

  use gfx::traits::{Stream, ToIndexSlice, ToSlice, FactoryExt};

  gfx_vertex!( Vertex {
      a_Pos@ pos: [f32; 2],
      a_Color@ color: [f32; 3],
  });

  pub fn start() {
    let mut client_state = ClientState::new();
    let client_params = params::query_client_params();
    let app_network = ClientNet::new(client_params.addr, client_params.server_addr);
    println!("Hello client!");
    app_network.send_event(ClientEvent::Connect);
    let (stdin_tx, stdin_rx) = channel();
    let mut last_sent = SteadyTime::now();
    let (mut mouse_x, mut mouse_y) = (0, 0);

    // Stdin handle
    thread::spawn (move || {
      let mut stdin = stdin();
      println!("type your messages");
      loop {
        let mut message = String::new();
        let _ = stdin.read_line(&mut message);
        let _ = stdin_tx.send(message);
      }
    });

    let (mut stream, mut device, mut factory) = gfx_window_glutin::init(
      glutin::Window::new().unwrap());
    stream.out.window.set_title("Space Coop Client");

    let vertex_data = [
        Vertex { pos: [ -0.5, -0.5 ], color: [1.0, 0.0, 0.0] },
        Vertex { pos: [  0.5, -0.5 ], color: [0.0, 1.0, 0.0] },
        Vertex { pos: [  0.0,  0.5 ], color: [0.0, 0.0, 1.0] },
    ];
    let mesh = factory.create_mesh(&vertex_data);
    let slice = mesh.to_slice(gfx::PrimitiveType::TriangleList);

    let program = {
        let vs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("../../shaders/triangle_120.glslv")),
            glsl_150: Some(include_bytes!("../../shaders/triangle_150.glslv")),
            .. gfx::ShaderSource::empty()
        };
        let fs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("../../shaders/triangle_120.glslf")),
            glsl_150: Some(include_bytes!("../../shaders/triangle_150.glslf")),
            .. gfx::ShaderSource::empty()
        };
        factory.link_program_source(vs, fs).unwrap()
    };
    let state = gfx::DrawState::new();

    'main: loop {

      // quit when Esc is pressed.
      for event in stream.out.window.poll_events() {
          match event {
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
              glutin::Event::MouseMoved((x, y)) => {
                mouse_x = x;
                mouse_y = y;
              },
              glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                app_network.send_event(ClientEvent::TryMove{x: mouse_x as f32, y: mouse_y as f32});
              },
              glutin::Event::Closed => break 'main,
              _ => {},
          }
      }

      stream.clear(gfx::ClearData {
          color: [0.3, 0.3, 0.3, 1.0],
          depth: 1.0,
          stencil: 0,
      });
      stream.draw(&gfx::batch::bind(&state, &mesh, slice.clone(), &program, &None))
            .unwrap();
      stream.present(&mut device);

      let possible_command = stdin_rx.try_recv().ok();
      possible_command.map (|message| {app_network.send_event(ClientEvent::Chat{message: message});});

      let now = SteadyTime::now();
      if (now - last_sent).num_seconds() > 1 {
        last_sent = now;
        app_network.send_event(ClientEvent::KeepAlive);
      }

      let events = app_network.get_events();
      events
        .into_iter()
        .foreach(|event| {
          match event {
            ServerEvent::Connected => println!("Connected"),
            ServerEvent::NotConnected => println!("Not Connected"),
            ServerEvent::Chatted {subject, message} => println!("{}: {}", subject, message.trim()),
            ServerEvent::Moved { x, y } => {
              client_state.position = (x, y);
              println!("Moved to {:?}", (x, y))
            },
            _ => ()
          }
        })
    }
    //app_network.send_event(ClientEvent::Disconnect);
  }
}
