pub mod shader;
pub mod model;
pub mod primitive;

use itertools::Itertools;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

pub use gfx_app::{ColorFormat, DepthFormat};
use gfx_app::shade;
use gfx::handle::{Sampler, ShaderResourceView};

use cgmath::{Matrix4, Quaternion, AffineMatrix3};


use common::world::{ClientWorld, PhysicalAspect};

use client::renderer::Renderer;
use client::renderer::opengl::primitive::Locals;
use client::renderer::opengl::primitive::pipe::{Data, Meta};

/**
 * A renderer using an OpenGl window to draw the state of the world
 */
pub struct OpenGlRenderer {
  pso: gfx::PipelineState<gfx_device_gl::Resources, Meta>,
  data: Data<gfx_device_gl::Resources>,
  box_color: (ShaderResourceView<gfx_device_gl::Resources, [f32;4]>, Sampler<gfx_device_gl::Resources>),
  ground_color: (ShaderResourceView<gfx_device_gl::Resources, [f32;4]>, Sampler<gfx_device_gl::Resources>),
  slice: gfx::Slice<gfx_device_gl::Resources>,

  proj: Matrix4<f32>,

  encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  window: glutin::Window,
  device: gfx_device_gl::Device,
}

impl OpenGlRenderer {
  pub fn new() -> OpenGlRenderer {
    use gfx::traits::FactoryExt;
    use gfx::Factory;
    use cgmath;
    use cgmath::{Point3, Vector3};
    use cgmath::{Transform, AffineMatrix3};

    let builder = glutin::WindowBuilder::new()
      .with_title("Space Coop".to_owned())
      .with_dimensions(1024, 768)
      .with_vsync();
    let (window, device, mut factory, main_color, main_depth) =
      gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let (width, height) = window.get_inner_size().unwrap();

    let aspect_ratio = width as f32 / height as f32;

    let backend = shade::Backend::Glsl(device.get_info().shading_language);

    let shader = shader::constants::cube_shader();

    let model = model::constants::cube();

    let (vbuf, slice) = factory.create_vertex_buffer_indexed(model.vertices.as_slice(), model.indices.as_slice());

    let box_texels = [[0xC0, 0xA0, 0x20, 0x00]];
    let (_, box_texture_view) = factory.create_texture_const::<gfx::format::Rgba8>(
        gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&box_texels]
        ).unwrap();

    let ground_texels = [[0xA0, 0xA0, 0xC0, 0x00]];
    let (_, ground_texture_view) = factory.create_texture_const::<gfx::format::Rgba8>(
        gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&ground_texels]
        ).unwrap();

    let sinfo = gfx::tex::SamplerInfo::new(
        gfx::tex::FilterMethod::Bilinear,
        gfx::tex::WrapMode::Clamp);

    let pso = factory.create_pipeline_simple(
        shader.vertex.select(backend).unwrap(),
        shader.fragment.select(backend).unwrap(),
        gfx::state::CullFace::Back,
        primitive::pipe::new()
    ).unwrap();

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), aspect_ratio, 1.0, 400.0);

    let data = primitive::pipe::Data {
        vbuf: vbuf.clone(),
        transform: (proj * view.mat).into(), // Totally useless value, is overwritten
        locals: factory.create_constant_buffer(1),
        color: (box_texture_view.clone(), factory.create_sampler(sinfo)),
        out_color: main_color.clone(),
        out_depth: main_depth.clone(),
    };

    OpenGlRenderer {
      pso: pso,
      data: data,
      slice: slice,
      box_color: (box_texture_view.clone(), factory.create_sampler(sinfo)),
      ground_color: (ground_texture_view.clone(), factory.create_sampler(sinfo)),

      proj: proj,

      encoder: encoder,
      window: window,
      device: device,
    }
  }

  pub fn mut_window(&mut self) -> &mut glutin::Window {
    &mut self.window
  }


  pub fn render_model(&mut self, physical_aspect: &PhysicalAspect, view: &AffineMatrix3<f32>) {
    let (x,y,z) = physical_aspect.pos;
    let model =
      // Minor hack to offset rendering for 1/2 height of cube to make bounce look good
      Matrix4::new(1.0, 0.0, 0.0 , 0.0,
                   0.0, 1.0, 0.0,  0.0,
                   0.0, 0.0, 1.0, 0.0,
                   x, y, (z + 1.0), 1.0);
    self.data.transform = (self.proj * view.mat * model).into();

    self.encoder.draw(&self.slice, &self.pso, &self.data);
  }
}

impl Renderer for OpenGlRenderer {
  fn render_world(&mut self, world_opt: &Option<&ClientWorld>, camera_pos: &(f32, f32, f32), _: &Quaternion<f32>) {
    use cgmath::{Transform, Matrix4, AffineMatrix3};
    use cgmath::{Point3, Vector3};

    let camera_focus = world_opt
      .and_then(|world| world.own_entity.and_then(|ent_uuid| world.physical.get(&ent_uuid)))
      .map(|physical| physical.pos.clone())
      .unwrap_or((0.0,0.0,0.0));

    let locals = Locals { transform: self.data.transform };
    self.encoder.update_constant_buffer(&self.data.locals, &locals);
    self.encoder.clear(&self.data.out_color, [0.1, 0.2, 0.3, 1.0]);
    self.encoder.clear_depth(&self.data.out_depth, 1.0);

    // Move the desired camera pos up by the ent pos
    let camera_adj_pos = (camera_focus.0 + camera_pos.0, camera_focus.1 + camera_pos.1, camera_focus.2 + camera_pos.2);

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(camera_adj_pos.0, camera_adj_pos.1, camera_adj_pos.2),
        Point3::new(camera_focus.0, camera_focus.1, camera_focus.2),
        Vector3::unit_z(),
    );
    self.data.color = self.ground_color.clone();

    let model =
      Matrix4::new(10.0, 0.0, 0.0 , 0.0,
                   0.0, 10.0, 0.0,  0.0,
                   0.0, 0.0, 0.01, 0.0,
                   0.0, 0.0, 0.0, 1.0);
    self.data.transform = (self.proj * view.mat * model).into();

    self.encoder.draw(&self.slice, &self.pso, &self.data);


    self.data.color = self.box_color.clone();
    // Blot each entity
    if world_opt.is_some() {
      let world = world_opt.unwrap();
      world.entities.iter().foreach(|uuid| {
        match (world.physical.get(uuid), world.rendered.get(uuid), world.disabled.contains(uuid)) {
          // TODO: use rendered_aspect to determine model
          (Some(physical_aspect), Some(_), false) => {
            self.render_model(physical_aspect, &view)
          }
          _ => {}
        }
      });
    }

    self.encoder.flush(&mut self.device);

    self.window.swap_buffers().unwrap();
    self.device.cleanup();
  }
}
