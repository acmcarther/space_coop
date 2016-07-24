pub mod shader;
pub mod model;
pub mod primitive;

use std::collections::HashMap;

use gfx::traits::FactoryExt;
use gfx;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

use cgmath::{Matrix4, Rad};
use cgmath::Euler;
use cgmath::Transform;
use cgmath::SquareMatrix;
use cgmath::Matrix;

use common::world::{PhysicalAspect, RenderAspect};
use common::model::ModelType;

use renderer::opengl::primitive::{ColorFormat, DepthFormat};
use renderer::opengl::primitive::pipe::{Data, Meta};
use gfx::handle::{DepthStencilView, RenderTargetView};
use cgmath;

/**
 * A renderer using an OpenGl window to draw the state of the world
 */
pub struct OpenGlRenderer {
  pso: gfx::PipelineState<gfx_device_gl::Resources, Meta>,
  models: HashMap<ModelType,
                  (Data<gfx_device_gl::Resources>, gfx::Slice<gfx_device_gl::Resources>)>,
  proj: Matrix4<f32>,
  view: Matrix4<f32>,
}

impl OpenGlRenderer {
  pub fn new(mut factory: gfx_device_gl::Factory,
             main_color: RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
             main_depth: DepthStencilView<gfx_device_gl::Resources, DepthFormat>)
             -> OpenGlRenderer {
    use gfx::traits::FactoryExt;
    use gfx::Factory;

    let shader = shader::constants::cube_shader();

    let box_texels = [[0xC0, 0xA0, 0x20, 0x00]];
    let (_, box_texture_view) =
      factory.create_texture_const::<ColorFormat>(gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&box_texels]) .unwrap();

    let set = factory.create_shader_set(shader.get_vertex(), shader.get_fragment()).unwrap();
    let pso = factory.create_pipeline_state(&set,
                             gfx::Primitive::TriangleList,
                             gfx::state::Rasterizer::new_fill().with_cull_back(),
                             primitive::pipe::new())
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
      pso: pso,
      models: models,
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
    encoder.draw(&slice, &self.pso, data);
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

    let model = Matrix4::new(10.0,
                             0.0,
                             0.0,
                             0.0,
                             0.0,
                             10.0,
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
      encoder.draw(&slice, &self.pso, data);
    }
  }
}
