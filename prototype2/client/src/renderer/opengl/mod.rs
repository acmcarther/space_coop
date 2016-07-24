pub mod shader;
pub mod model;
pub mod primitive;

use std::collections::HashMap;

use gfx::traits::FactoryExt;
use gfx;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

use gfx::handle::{Sampler, ShaderResourceView};

use cgmath::{Matrix4, Rad};
use cgmath::Euler;
use cgmath::Transform;
use cgmath::SquareMatrix;
use cgmath::Matrix;

use common::world::PhysicalAspect;

use renderer::opengl::primitive::{ColorFormat, DepthFormat, Locals};
use renderer::opengl::primitive::pipe::{Data, Meta};
use gfx::handle::{DepthStencilView, RenderTargetView};
use cgmath;

/**
 * A renderer using an OpenGl window to draw the state of the world
 */
pub struct OpenGlRenderer {
  pso: gfx::PipelineState<gfx_device_gl::Resources, Meta>,
  box_color: (ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
              Sampler<gfx_device_gl::Resources>),
  ground_color: (ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
                 Sampler<gfx_device_gl::Resources>),
  models: HashMap<&'static str,
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
    use cgmath::{Point3, Vector3};
    use cgmath::Transform;

    let shader = shader::constants::cube_shader();

    let box_texels = [[0xC0, 0xA0, 0x20, 0x00]];
    let (_, box_texture_view) =
      factory.create_texture_const::<ColorFormat>(gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&box_texels]) .unwrap();

    let ground_texels = [[0xA0, 0xA0, 0xC0, 0x00]];
    let (_, ground_texture_view) =
      factory.create_texture_const::<ColorFormat>(gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&ground_texels]) .unwrap();

    let sinfo = gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Bilinear,
                                           gfx::tex::WrapMode::Clamp);

    let set = factory.create_shader_set(shader.get_vertex(), shader.get_fragment()).unwrap();
    let pso = factory.create_pipeline_state(&set,
                             gfx::Primitive::TriangleList,
                             gfx::state::Rasterizer::new_fill().with_cull_back(),
                             primitive::pipe::new())
      .unwrap();

    let dummy_view: Matrix4<f32> = Transform::look_at(Point3::new(1.5f32, -5.0, 3.0),
                                                      Point3::new(0f32, 0.0, 0.0),
                                                      Vector3::unit_z());

    // cube
    let cube_model = model::constants::cube();
    let (cube_vbuf, cube_slice) =
      factory.create_vertex_buffer_with_slice(cube_model.vertices.as_slice(),
                                              cube_model.indices.as_slice());
    let cube_data = primitive::pipe::Data {
      vbuf: cube_vbuf.clone(),
      camera_pv: (dummy_view).into(), // Totally useless value, is overwritten
      obj_to_world: (dummy_view).into(), // Totally useless value, is overwritten
      norm_to_world: (dummy_view).into(), // Totally useless value, is overwritten
      light_pos: [0.0, 0.0, 0.0], // Totally useless value, is overwritten
      camera_pos: [0.0, 0.0, 0.0], // Totally useless value, is overwritten
      locals: factory.create_constant_buffer(1),
      color: (box_texture_view.clone(), factory.create_sampler(sinfo)), /* overwritten on render
                                                                         * as well? */
      out_color: main_color.clone(),
      out_depth: main_depth.clone(),
    };

    // icosphere 1
    let ico_model = model::constants::icosphere(2);
    let (ico_vbuf, ico_slice) =
      factory.create_vertex_buffer_with_slice(ico_model.vertices.as_slice(),
                                              ico_model.indices.as_slice());
    let ico_data = primitive::pipe::Data {
      vbuf: ico_vbuf.clone(),
      camera_pv: (dummy_view).into(), // Totally useless value, is overwritten
      obj_to_world: (dummy_view).into(), // Totally useless value, is overwritten
      norm_to_world: (dummy_view).into(), // Totally useless value, is overwritten
      light_pos: [0.0, 0.0, 0.0], // Totally useless value, is overwritten
      camera_pos: [0.0, 0.0, 0.0], // Totally useless value, is overwritten
      locals: factory.create_constant_buffer(1),
      color: (box_texture_view.clone(), factory.create_sampler(sinfo)), /* overwritten on render
                                                                         * as well? */
      out_color: main_color.clone(),
      out_depth: main_depth.clone(),
    };

    let mut models = HashMap::new();

    models.insert("cube", (cube_data, cube_slice));
    models.insert("sphere", (ico_data, ico_slice));

    OpenGlRenderer {
      pso: pso,
      box_color: (box_texture_view.clone(), factory.create_sampler(sinfo)),
      ground_color: (ground_texture_view.clone(), factory.create_sampler(sinfo)),
      models: models,
      proj: dummy_view.clone(),
      view: dummy_view,
    }
  }

  pub fn render_model(&mut self,
                      encoder: &mut gfx::Encoder<gfx_device_gl::Resources,
                                                 gfx_device_gl::CommandBuffer>,
                      window: &mut glutin::Window,
                      physical_aspect: &PhysicalAspect,
                      camera_adj_pos: (f32, f32, f32)) {
    let light_pos: [f32; 3] = [1.0, 1.0, 5.0];
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

    let &mut (ref mut data, ref slice) = self.models.get_mut("sphere").unwrap();

    data.camera_pv = (self.proj * self.view).into();
    data.obj_to_world = (model).into();
    data.norm_to_world = (model).invert().unwrap().transpose().into();
    data.color = self.box_color.clone();
    data.light_pos = light_pos.clone();
    data.camera_pos = [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2];
    let locals = Locals {
      obj_to_world: data.obj_to_world,
      camera_pv: data.camera_pv,
      norm_to_world: (model).invert().unwrap().transpose().into(),
      light_pos: light_pos,
      camera_pos: [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2],
    };
    gfx_window_glutin::update_views(window, &mut data.out_color, &mut data.out_depth);
    encoder.update_constant_buffer(&data.locals, &locals);
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
      let &mut (ref mut data, _) = self.models.get_mut("cube").unwrap();
      encoder.clear(&data.out_color, [0.1, 0.2, 0.3, 1.0]);
      encoder.clear_depth(&data.out_depth, 1.0);

      gfx_window_glutin::update_views(window, &mut data.out_color, &mut data.out_depth);

      // Get our encoder from the main thread
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

    let light_pos: [f32; 3] = [1.0, 1.0, 5.0];
    {
      let &mut (ref mut data, ref slice) = self.models.get_mut("cube").unwrap();
      data.color = self.ground_color.clone();
      data.camera_pv = (self.proj * self.view).into();
      data.obj_to_world = (model).into();
      data.norm_to_world = (model).invert().unwrap().transpose().into();
      data.light_pos = light_pos.clone();
      data.camera_pos = [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2];
      let locals = Locals {
        obj_to_world: data.obj_to_world,
        camera_pv: data.camera_pv,
        norm_to_world: (self.view * model).invert().unwrap().transpose().into(),
        light_pos: light_pos.clone(),
        camera_pos: [camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2],
      };
      encoder.update_constant_buffer(&data.locals, &locals);
      encoder.draw(&slice, &self.pso, data);
    }
  }
}
