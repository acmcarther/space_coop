pub mod shader;
pub mod model;
pub mod primitive3d;

use std::collections::HashMap;

use gfx;
use gfx_text;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

use debug;
use pause;
use console;
use cgmath::{Matrix4, Rad};
use cgmath::Euler;
use cgmath::Transform;
use cgmath::SquareMatrix;
use cgmath::Matrix;

use common::world::{PhysicalAspect, RenderAspect};
use common::model::ModelType;

use gfx::handle::{DepthStencilView, RenderTargetView};
use cgmath;

/**
 * A renderer using an OpenGl window to draw the state of the world
 */
pub struct OpenGlRenderer {
  pso_3d: gfx::PipelineState<gfx_device_gl::Resources, primitive3d::pipe::Meta>,
  models: HashMap<ModelType,
                  (primitive3d::pipe::Data<gfx_device_gl::Resources>,
                   gfx::Slice<gfx_device_gl::Resources>)>,
  main_color: RenderTargetView<gfx_device_gl::Resources, primitive3d::ColorFormat>,
  main_depth: DepthStencilView<gfx_device_gl::Resources, primitive3d::DepthFormat>,
  text_renderer: gfx_text::Renderer<gfx_device_gl::Resources, gfx_device_gl::Factory>,
  proj: Matrix4<f32>,
  view: Matrix4<f32>,
}

impl OpenGlRenderer {
  pub fn new(mut factory: gfx_device_gl::Factory,
             main_color: RenderTargetView<gfx_device_gl::Resources, primitive3d::ColorFormat>,
             main_depth: DepthStencilView<gfx_device_gl::Resources, primitive3d::DepthFormat>)
             -> OpenGlRenderer {
    use gfx::traits::FactoryExt;
    use gfx::Factory;

    let cube_shader = shader::constants::cube_shader();

    let box_texels = [[0xC0, 0xA0, 0x20, 0x00]];
    let (_, box_texture_view) =
      factory.create_texture_const::<primitive3d::ColorFormat>(gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&box_texels]) .unwrap();

    let set = factory.create_shader_set(cube_shader.get_vertex(), cube_shader.get_fragment())
      .unwrap();
    let pso_3d = factory.create_pipeline_state(&set,
                             gfx::Primitive::TriangleList,
                             gfx::state::Rasterizer::new_fill().with_cull_back(),
                             primitive3d::pipe::new())
      .unwrap();

    let mut models = HashMap::new();
    let (cube_slice, cube_data) = model::build_cube(&mut factory,
                                                    box_texture_view.clone(),
                                                    main_color.clone(),
                                                    main_depth.clone());
    let (ico0_slice, ico0_data) = model::build_icosphere(0,
                                                         &mut factory,
                                                         box_texture_view.clone(),
                                                         main_color.clone(),
                                                         main_depth.clone());;
    let (ico1_slice, ico1_data) = model::build_icosphere(1,
                                                         &mut factory,
                                                         box_texture_view.clone(),
                                                         main_color.clone(),
                                                         main_depth.clone());
    let (ico2_slice, ico2_data) = model::build_icosphere(2,
                                                         &mut factory,
                                                         box_texture_view.clone(),
                                                         main_color.clone(),
                                                         main_depth.clone());
    let (ico3_slice, ico3_data) = model::build_icosphere(3,
                                                         &mut factory,
                                                         box_texture_view.clone(),
                                                         main_color.clone(),
                                                         main_depth.clone());

    models.insert(ModelType::Cube, (cube_data, cube_slice));
    models.insert(ModelType::Icosphere0, (ico0_data, ico0_slice));
    models.insert(ModelType::Icosphere1, (ico1_data, ico1_slice));
    models.insert(ModelType::Icosphere2, (ico2_data, ico2_slice));
    models.insert(ModelType::Icosphere3, (ico3_data, ico3_slice));

    OpenGlRenderer {
      pso_3d: pso_3d,
      models: models,
      main_color: main_color,
      main_depth: main_depth,
      text_renderer: gfx_text::new(factory).build().unwrap(),
      proj: Matrix4::identity(), // Overwritten on first call to render_world
      view: Matrix4::identity(), // Overwritten on first call to render_world
    }
  }

  pub fn render_model(&mut self,
                      encoder: &mut gfx::Encoder<gfx_device_gl::Resources,
                                                 gfx_device_gl::CommandBuffer>,
                      window: &mut glutin::Window,
                      physical_aspect: &PhysicalAspect,
                      render_aspect: &RenderAspect,
                      camera_adj_pos: (f32, f32, f32)) {
    let light_pos: [f32; 4] = [0.0, 0.0, 5.0, 1.0];
    let (x, y, z) = physical_aspect.pos;
    let translation =
      // Minor hack to offset rendering for 1/2 height of cube to make bounce look good
      Matrix4::new(1.02, 0.0, 0.0 , 0.0,
                   0.0, 1.02, 0.0,  0.0,
                   0.0, 0.0, 1.02, 0.0,
                   x, y, (z + 1.0), 1.0);
    let (rx, ry, rz) = physical_aspect.ang;
    let rotation = Matrix4::from(Euler::new(Rad::new(-rx), Rad::new(-rz), Rad::new(-ry)));
    let model = translation.concat(&rotation);

    let &mut (ref mut data, ref slice) = self.models.get_mut(&render_aspect.model).unwrap();

    data.camera_pv = (self.proj * self.view).into();
    data.obj_to_world = (model).into();
    data.norm_to_world = (model).invert().unwrap().transpose().into();
    data.light_pos = light_pos.clone();
    data.camera_pos = [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2];
    gfx_window_glutin::update_views(window, &mut data.out_color, &mut data.out_depth);
    // encoder.update_constant_buffer(&data.locals, &locals);
    encoder.draw(&slice, &self.pso_3d, data);
  }

  pub fn render_ui(&mut self,
                   encoder: &mut gfx::Encoder<gfx_device_gl::Resources,
                                              gfx_device_gl::CommandBuffer>,
                   window: &mut glutin::Window,
                   debug_msg: &debug::DebugMessage,
                   console_buffer: &console::CommandBuffer,
                   console_log: &console::ConsoleLog,
                   pause_state: &pause::PauseState) {
    use itertools::Itertools;

    let &debug::DebugMessage(ref message) = debug_msg;
    gfx_window_glutin::update_views(window, &mut self.main_color, &mut self.main_depth);
    // Add some text 10 pixels down and right from the top left screen corner.
    self.text_renderer.add(message, // Text to add
                           [10, 10], // Position
                           [0.9, 0.9, 0.9, 1.0] /* Text color */);

    if *pause_state == pause::PauseState::Paused {
      self.text_renderer.add("Menu is open \n ", // Text to add
                             [50, 50], // Position
                             [0.9, 0.9, 0.9, 1.0] /* Text color */);

      // TODO: display cursor
      // let &console::CommandCursor(ref cursor) = console_cursor;
      let &console::CommandBuffer(ref msg) = console_buffer;
      self.text_renderer.add(msg, // Text to add
                             [50, 300], // Position
                             [0.9, 0.9, 0.9, 1.0] /* Text color */);
      console_log.list(10).into_iter().enumerate().foreach(|(idx, element)| {
        let y_pos = 280 - (idx as i32) * 20; // Scroll up the screen
        self.text_renderer.add(element, [50, y_pos], [0.9, 0.9, 0.9, 1.0]);
      });
    }

    // Draw text.
    self.text_renderer.draw(encoder, &mut self.main_color).unwrap();
  }

  pub fn render_world(&mut self,
                      encoder: &mut gfx::Encoder<gfx_device_gl::Resources,
                                                 gfx_device_gl::CommandBuffer>,
                      window: &mut glutin::Window,
                      camera_pos: &(f32, f32, f32),
                      camera_target: Option<(f32, f32, f32)>) {
    use cgmath::{Matrix4, Transform};
    use cgmath::{Point3, Vector3};

    // Calculate projection matrix
    let (width, height) = window.get_inner_size().unwrap();
    let aspect_ratio = width as f32 / height as f32;
    self.proj = cgmath::perspective(cgmath::deg(45.0f32), aspect_ratio, 1.0, 400.0);

    // TODO:(we need the generation for this), for now just aim at ground
    let camera_focus = camera_target.unwrap_or((0.0, 0.0, 0.0));

    {
      let &mut (ref mut data, _) = self.models.get_mut(&ModelType::Cube).unwrap();
      encoder.clear(&data.out_color, [0.1, 0.2, 0.3, 1.0]);
      encoder.clear_depth(&data.out_depth, 1.0);

      gfx_window_glutin::update_views(window, &mut data.out_color, &mut data.out_depth);
    }

    // Move the desired camera pos up by the ent pos
    let camera_adj_pos =
      (camera_focus.0 + camera_pos.0, camera_focus.1 + camera_pos.1, camera_focus.2 + camera_pos.2);

    self.view =
      Transform::look_at(Point3::new(camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2),
                         Point3::new(camera_focus.0, camera_focus.1, camera_focus.2),
                         Vector3::unit_z());

    let model = Matrix4::new(30.0,
                             0.0,
                             0.0,
                             0.0,
                             0.0,
                             30.0,
                             0.0,
                             0.0,
                             0.0,
                             0.0,
                             0.01,
                             0.0,
                             0.0,
                             0.0,
                             0.0,
                             1.0);

    let light_pos: [f32; 4] = [0.0, 0.0, 5.0, 1.0];
    {
      let &mut (ref mut data, ref slice) = self.models.get_mut(&ModelType::Cube).unwrap();
      data.camera_pv = (self.proj * self.view).into();
      data.obj_to_world = (model).into();
      data.norm_to_world = (model).invert().unwrap().transpose().into();
      data.light_pos = light_pos.clone();
      data.camera_pos = [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2];
      // encoder.update_constant_buffer(&data.locals, &locals);
      encoder.draw(&slice, &self.pso_3d, data);
    }
  }
}
