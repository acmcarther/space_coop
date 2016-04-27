use itertools::Itertools;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

pub use gfx_app::{ColorFormat, DepthFormat};
use gfx_app::{self, shade};

use cgmath::{Matrix4, Quaternion};

use common::world::ClientWorld;

// Declare the vertex format suitable for drawing.
// Notice the use of FixedPoint.
gfx_vertex_struct!( Vertex {
  pos: [i8; 4] = "a_Pos",
  tex_coord: [i8; 2] = "a_TexCoord",
});

impl Vertex {
  fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
    Vertex {
      pos: [p[0], p[1], p[2], 1],
      tex_coord: t,
    }
  }
}

gfx_constant_struct!( Locals {
  transform: [[f32; 4]; 4] = "u_Transform",
});

gfx_pipeline!( pipe {
  vbuf: gfx::VertexBuffer<Vertex> = (),
  transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
  locals: gfx::ConstantBuffer<Locals> = "Locals",
  color: gfx::TextureSampler<[f32; 4]> = "t_Color",
  out_color: gfx::RenderTarget<ColorFormat> = "Target0",
  out_depth: gfx::DepthTarget<DepthFormat> =
    gfx::preset::depth::LESS_EQUAL_WRITE,
});

use client::renderer::opengl::pipe::{Data, Meta};

pub struct OpenGlRenderer {
  pso: gfx::PipelineState<gfx_device_gl::Resources, Meta>,
  data: Data<gfx_device_gl::Resources>,
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


    let vs = gfx_app::shade::Source {
        glsl_120: include_bytes!("shader/cube_120.glslv"),
        glsl_150: include_bytes!("shader/cube_150.glslv"),
        .. gfx_app::shade::Source::empty()
    };
    let ps = gfx_app::shade::Source {
        glsl_120: include_bytes!("shader/cube_120.glslf"),
        glsl_150: include_bytes!("shader/cube_150.glslf"),
        .. gfx_app::shade::Source::empty()
    };

    ///////////////////////////////////////////////////////////////////////////////////////////
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

    let index_data: &[u16] = &[
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];


    ///////////////////////////////////////////////////////////////////////////////////////////
    let (vbuf, slice) = factory.create_vertex_buffer_indexed(&vertex_data, index_data);

    let texels = [[0xC0, 0xA0, 0x20, 0x00]];
    let (_, texture_view) = factory.create_texture_const::<gfx::format::Rgba8>(
        gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&texels]
        ).unwrap();

    let sinfo = gfx::tex::SamplerInfo::new(
        gfx::tex::FilterMethod::Bilinear,
        gfx::tex::WrapMode::Clamp);

    let pso = factory.create_pipeline_simple(
        vs.select(backend).unwrap(),
        ps.select(backend).unwrap(),
        gfx::state::CullFace::Back,
        pipe::new()
    ).unwrap();

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), aspect_ratio, 1.0, 400.0);

    let data = pipe::Data {
        vbuf: vbuf.clone(),
        transform: (proj * view.mat).into(), // Totally useless value, is overwritten
        locals: factory.create_constant_buffer(1),
        color: (texture_view.clone(), factory.create_sampler(sinfo)),
        out_color: main_color.clone(),
        out_depth: main_depth.clone(),
    };


    OpenGlRenderer {
      pso: pso,
      data: data,
      slice: slice,

      proj: proj,

      encoder: encoder,
      window: window,
      device: device,
    }
  }

  pub fn mut_window(&mut self) -> &mut glutin::Window {
    &mut self.window
  }

  pub fn render_world(&mut self, world_opt: &Option<&ClientWorld>, camera_pos: &(f32, f32, f32), _: &Quaternion<f32>) {
    use cgmath::{Transform, Matrix4, AffineMatrix3};
    use cgmath::{Point3, Vector3};

    let locals = Locals { transform: self.data.transform };
    self.encoder.update_constant_buffer(&self.data.locals, &locals);
    self.encoder.clear(&self.data.out_color, [0.1, 0.2, 0.3, 1.0]);
    self.encoder.clear_depth(&self.data.out_depth, 1.0);

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(camera_pos.0, camera_pos.1, camera_pos.2),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );

    // Blot each entity
    if world_opt.is_some() {
      let world = world_opt.unwrap();
      world.entities.iter().foreach(|uuid| {
        match (world.physical.get(uuid), world.rendered.get(uuid), world.disabled.contains(uuid)) {
          // TODO: use rendered_aspect to determine model
          (Some(physical_aspect), Some(_), false) => {
            let (x,y,z) = physical_aspect.pos;
            let model =
              Matrix4::new(1.0, 0.0, 0.0 , 0.0,
                           0.0, 1.0, 0.0,  0.0,
                           0.0, 0.0, 1.0, 0.0,
                           x, y, z, 1.0);
            self.data.transform = (self.proj * view.mat * model).into();

            self.encoder.draw(&self.slice, &self.pso, &self.data);
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
