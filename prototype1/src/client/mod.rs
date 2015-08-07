pub use self::client::{start};

mod client {
  use params;

  use app_net::ClientNet;
  use events::{
    ClientEvent,
    ServerEvent,
    EntEvent,
  };
  use state::{
    ClientState,
    Primitive
  };

  use std::thread;
  use std::io::stdin;
  use std::sync::mpsc::channel;
  use std::marker::PhantomData;
  use std::collections::hash_map::{Entry};

  use itertools::Itertools;
  use time::SteadyTime;

  use glutin;
  use gfx_window_glutin;
  use gfx;
  use cgmath;

  use cgmath::FixedArray;
  use cgmath::{Matrix, Point, Point3, Vector, Vector3, Vector4, Matrix4};
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
    let (eye_x, eye_y, eye_z) = (1.5f32, -5.0, 3.0);
    let mut view: AffineMatrix3<f32> = Transform::look_at(
        &Point3::new(eye_x, eye_y, eye_z),
        &Point3::new(0f32, 0.0, 0.0),
        &Vector3::unit_z(),
    );

    let proj = cgmath::perspective(cgmath::deg(45.0f32),
                                   stream.get_aspect_ratio(), 1.0, 400.0);

    let origin_primitive = Primitive { pos: (0.0, 0.0), color: (0, 255, 0) };
    let origin_entity = (&0u8, &origin_primitive);

    // loooop
    'main: loop {

      // Event handling
      for event in stream.out.window.poll_events() {
          match event {
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Escape)) => break 'main,
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::A)) => {
                println!("own id {:?}", client_state.own_id);
                println!("entities {:?}", client_state.entities);
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (x, y) = primitive.pos;
                    app_network.send_event(ClientEvent::MoveSelf{x: x + 1.0, y: y});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::W)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (x, y) = primitive.pos;
                    app_network.send_event(ClientEvent::MoveSelf{x: x, y: y + 1.0});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::S)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (x, y) = primitive.pos;
                    app_network.send_event(ClientEvent::MoveSelf{x: x, y: y - 1.0});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::D)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (x, y) = primitive.pos;
                    app_network.send_event(ClientEvent::MoveSelf{x: x - 1.0, y: y});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::U)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r.saturating_add(1), g: g, b: b});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::J)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r.saturating_sub(1), g: g, b: b});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::I)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r, g: g.saturating_add(1), b: b});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::K)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r, g: g.saturating_sub(1), b: b});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::O)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r, g: g, b: b.saturating_add(1)});
                  });
                });
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::L)) => {
                client_state.own_id.map(|id| {
                  client_state.entities.get(&id).map(|primitive| {
                    let (r, g, b) = primitive.color;
                    app_network.send_event(ClientEvent::SetOwnColor{r: r, g: g, b: b.saturating_sub(1)});
                  });
                });
              },
              glutin::Event::MouseMoved((x, y)) => {
                mouse_x = x;
                mouse_y = y;
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::Z)) => {
                client_state.zoom_level = client_state.zoom_level.saturating_add(1);
              },
              glutin::Event::KeyboardInput(_, _, Some(glutin::VirtualKeyCode::X)) => {
                client_state.zoom_level = client_state.zoom_level.saturating_sub(1);
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
                //app_network.send_event(ClientEvent::TryMove{x: mouse_x as f32, y: mouse_y as f32});
              },
              glutin::Event::Closed => break 'main,
              _ => {},
          }
      }
      view = Transform::look_at(
          &Point3::new(eye_x, eye_y, eye_z).mul_s(client_state.zoom_level as f32),
          &Point3::new(0f32, 0.0, 0.0),
          &Vector3::unit_z(),
      );

      stream.clear(gfx::ClearData {
          color: [0.3, 0.3, 0.3, 1.0],
          depth: 1.0,
          stencil: 0,
      });
      client_state.entities.iter()
        .chain(vec![origin_entity].into_iter())
        .foreach( |(_, primitive): (&u8, &Primitive)| {
            let texture = factory.create_texture_rgba8(1, 1).unwrap();
            let (r, g, b) = primitive.color.clone();
            let (x, y) = primitive.pos.clone();
            let model_mat =
              Matrix4::new(1.0, 0.0, 0.0 , 0.0,
                           0.0, 1.0, 0.0,  0.0,
                           0.0, 0.0, 1.0, 0.0,
                           x, y, 0.0, 1.0);
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
                transform: proj.mul_m(&view.mat.mul_m(&model_mat)).into_fixed(),
                color: (texture.clone(), Some(sampler)),
                _r: PhantomData,
            };

            let mut batch = gfx::batch::Full::new(mesh.clone(), program.clone(), data).unwrap();
            batch.slice = index_data.to_slice(&mut factory, gfx::PrimitiveType::TriangleList);
            batch.state = batch.state.depth(gfx::state::Comparison::LessEqual, true);

            stream.draw(&batch).unwrap();
      });
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
            ServerEvent::Connected { eid } => {
              println!("Connected as entId: {}", eid);
              client_state.own_id = Some(eid.clone());
              client_state.entities.insert(eid, Primitive {pos: (0.0, 0.0), color: (200, 200, 200)});
            },
            ServerEvent::NotConnected => println!("Not Connected"),
            ServerEvent::Chatted {subject, message} => println!("{}: {}", subject, message.trim()),
            ServerEvent::EntEvent {eid, event} => {
              match event {
                EntEvent::Spawned  => {client_state.entities.insert(eid, Primitive {color: (200, 200, 200), pos: (0.0, 0.0)});},
                EntEvent::Moved { x, y } => {
                  println!("got an ent moved {:?}", (x, y));
                  let entity = client_state.entities.entry(eid).or_insert(Primitive{pos: (0.0, 0.0), color: (200, 200, 200)});
                  entity.pos = (x, y);
                },
                EntEvent::Recolored {r, g, b} => {
                  match client_state.entities.entry(eid) {
                    Entry::Occupied(mut value) => {
                      let primitive = value.get_mut();
                      primitive.color = (r, g, b);
                    },
                    _ => ()
                  };
                },
                EntEvent::Destroyed => {client_state.entities.remove(&eid);}
              };
            },
            _ => ()
          }
        })
    }
    //app_network.send_event(ClientEvent::Disconnect);
  }
}
