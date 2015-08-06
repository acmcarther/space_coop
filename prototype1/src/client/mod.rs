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
  use std::marker::PhantomData;

  use itertools::Itertools;
  use time::SteadyTime;

  use glutin;
  use gfx_window_glutin;
  use gfx;
  use cgmath;

  use cgmath::FixedArray;
  use cgmath::{Matrix, Point3, Vector, Vector3, Vector4, Matrix4};
  use cgmath::{Transform, AffineMatrix3};
  use gfx::attrib::Floater;
  use gfx::traits::{Factory, Stream, ToIndexSlice, ToSlice, FactoryExt};

  // Declare the vertex format suitable for drawing.
  // Notice the use of FixedPoint.
  gfx_vertex!( Vertex {
      a_Pos@ pos: [Floater<i8>; 3],
      a_TexCoord@ tex_coord: [Floater<u8>; 2],
  });

  impl Vertex {
      fn new(p: [i8; 3], t: [u8; 2]) -> Vertex {
          Vertex {
              pos: Floater::cast3(p),
              tex_coord: Floater::cast2(t),
          }
      }
  }

  // The shader_param attribute makes sure the following struct can be used to
  // pass parameters to a shader.
  gfx_parameters!( Params {
      u_Transform@ transform: [[f32; 4]; 4],
      t_Color@ color: gfx::shade::TextureParam<R>,
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

    // Glutin init
    let (mut stream, mut device, mut factory) = gfx_window_glutin::init(
      glutin::Window::new().unwrap());
    stream.out.window.set_title("Space Coop Client");

    // gfx init
    let vertex_data = [
        // top (0, 0, 1)
        Vertex::new([-1, -1,  1], [0, 0]),
        Vertex::new([ 1, -1,  1], [1, 0]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        Vertex::new([-1,  1,  1], [0, 1]),
        // bottom (0, 0, -1)
        Vertex::new([-1,  1, -1], [1, 0]),
        Vertex::new([ 1,  1, -1], [0, 0]),
        Vertex::new([ 1, -1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // right (1, 0, 0)
        Vertex::new([ 1, -1, -1], [0, 0]),
        Vertex::new([ 1,  1, -1], [1, 0]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        Vertex::new([ 1, -1,  1], [0, 1]),
        // left (-1, 0, 0)
        Vertex::new([-1, -1,  1], [1, 0]),
        Vertex::new([-1,  1,  1], [0, 0]),
        Vertex::new([-1,  1, -1], [0, 1]),
        Vertex::new([-1, -1, -1], [1, 1]),
        // front (0, 1, 0)
        Vertex::new([ 1,  1, -1], [1, 0]),
        Vertex::new([-1,  1, -1], [0, 0]),
        Vertex::new([-1,  1,  1], [0, 1]),
        Vertex::new([ 1,  1,  1], [1, 1]),
        // back (0, -1, 0)
        Vertex::new([ 1, -1,  1], [0, 0]),
        Vertex::new([-1, -1,  1], [1, 0]),
        Vertex::new([-1, -1, -1], [1, 1]),
        Vertex::new([ 1, -1, -1], [0, 1]),
    ];

    let mesh = factory.create_mesh(&vertex_data);

    let index_data: &[u8] = &[
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    let program = {
        let vs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("../../shaders/cube_120.glslv")),
            glsl_150: Some(include_bytes!("../../shaders/cube_150.glslv")),
            .. gfx::ShaderSource::empty()
        };
        let fs = gfx::ShaderSource {
            glsl_120: Some(include_bytes!("../../shaders/cube_120.glslf")),
            glsl_150: Some(include_bytes!("../../shaders/cube_150.glslf")),
            .. gfx::ShaderSource::empty()
        };
        factory.link_program_source(vs, fs).unwrap()
    };
    let (mut eye_x, mut eye_y, mut eye_z) = (1.5f32, -5.0, 3.0);
    let mut view: AffineMatrix3<f32> = Transform::look_at(
        &Point3::new(eye_x, eye_y, eye_z),
        &Point3::new(0f32, 0.0, 0.0),
        &Vector3::unit_z(),
    );

    let proj = cgmath::perspective(cgmath::deg(45.0f32),
                                   stream.get_aspect_ratio(), 1.0, 40.0);

    // loooop
    'main: loop {

      // Event handling
      for event in stream.out.window.poll_events() {
          match event {
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Q)) => {
                eye_x = eye_x - 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::A)) => {
                eye_x = eye_x + 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::W)) => {
                eye_y = eye_y - 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::S)) => {
                eye_y = eye_y + 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::E)) => {
                eye_z = eye_z - 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::D)) => {
                eye_z = eye_z + 0.1
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::U)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r.saturating_add(1), g: g, b: b});
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::J)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r.saturating_sub(1), g: g, b: b});
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::I)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r, g: g.saturating_add(1), b: b});
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::K)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r, g: g.saturating_sub(1), b: b});
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::O)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r, g: g, b: b.saturating_add(1)});
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::L)) => {
                let (r, g, b) = client_state.cube_color;
                app_network.send_event(ClientEvent::SetColor{r: r, g: g, b: b.saturating_sub(1)});
              },
              glutin::Event::MouseMoved((x, y)) => {
                mouse_x = x;
                mouse_y = y;
              },
              glutin::Event::MouseInput(glutin::ElementState::Pressed, glutin::MouseButton::Left) => {
                let (screen_x, screen_y) = stream.out.window.get_inner_size().unwrap();
                println!("screen {:?}", (screen_x, screen_y));
                let norm_x = 2.0 * (mouse_x as f32 - (0.5 * screen_x as f32))/(screen_x as f32);
                let norm_y = 2.0 * (mouse_y as f32 - (0.5 * screen_y as f32))/(screen_y as f32);
                let coord_vec = Vector4::new(norm_x, norm_y, -1.0, 1.0);
                println!("screen pos {:?}", coord_vec);
                let world_pos = (Matrix4::from(view) * proj).invert().unwrap().mul_v(&coord_vec);
                let result_vec = world_pos.truncate().div_s(world_pos.w);
                println!("world pos {:?}", result_vec);
                app_network.send_event(ClientEvent::TryMove{x: mouse_x as f32, y: mouse_y as f32});
              },
              glutin::Event::Closed => break 'main,
              _ => {},
          }
      }
      view = Transform::look_at(
          &Point3::new(eye_x, eye_y, eye_z),
          &Point3::new(0f32, 0.0, 0.0),
          &Vector3::unit_z(),
      );

      let texture = factory.create_texture_rgba8(1, 1).unwrap();
      let (r, g, b) = client_state.cube_color;
      factory.update_texture(
          &texture, &(*texture.get_info()).into(),
          &[r, g, b, 0x00u8],
          None).unwrap();

      let sampler = factory.create_sampler(
          gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Bilinear,
                                     gfx::tex::WrapMode::Clamp)
      );


      // Actual render
      let data = Params {
          transform: proj.mul_m(&view.mat).into_fixed(),
          color: (texture.clone(), Some(sampler)),
          _r: PhantomData,
      };

      let mut batch = gfx::batch::Full::new(mesh.clone(), program.clone(), data).unwrap();
      batch.slice = index_data.to_slice(&mut factory, gfx::PrimitiveType::TriangleList);
      batch.state = batch.state.depth(gfx::state::Comparison::LessEqual, true);

      stream.clear(gfx::ClearData {
          color: [0.3, 0.3, 0.3, 1.0],
          depth: 1.0,
          stencil: 0,
      });
      stream.draw(&batch).unwrap();
      stream.present(&mut device);

      // Networking
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
            ServerEvent::ColorIs {r, g, b} => {
              println!("coloris {:?}", (r, g, b));
              client_state.cube_color = (r, g, b)
            },
            _ => ()
          }
        })
    }
    //app_network.send_event(ClientEvent::Disconnect);
  }
}
