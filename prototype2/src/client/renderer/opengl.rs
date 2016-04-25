use common::world::ClientWorld;
use client::renderer::Renderer;

use gfx::traits::FactoryExt;
use gfx::Device;
use gfx;
use glutin;
use gfx_window_glutin;
use gfx_device_gl;

pub use gfx_app::{ColorFormat, DepthFormat};
use gfx_app;
use gfx_app::shade;
use gfx_app::DEFAULT_CONFIG;

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

  encoder: gfx::Encoder<gfx_device_gl::Resources, gfx_device_gl::CommandBuffer>,
  window: glutin::Window,
  device: gfx_device_gl::Device,
}

impl OpenGlRenderer {
  pub fn new() -> OpenGlRenderer {
    use gfx::traits::FactoryExt;
    use gfx::traits::Device;
    use gfx::Factory;
    use cgmath;
    use cgmath::{Point3, Vector3};
    use cgmath::{Transform, AffineMatrix3};

    let builder = glutin::WindowBuilder::new()
      .with_title("Space Coop".to_owned())
      .with_dimensions(1024, 768)
      .with_vsync();
    let (window, mut device, mut factory, main_color, main_depth) =
      gfx_window_glutin::init::<ColorFormat, DepthFormat>(builder);
    let mut encoder: gfx::Encoder<_, _> = factory.create_command_buffer().into();

    let (width, height) = window.get_inner_size().unwrap();

    let aspect_ratio = width as f32 / height as f32;

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

    let texels = [[0x20, 0xA0, 0xC0, 0x00]];
    let (_, texture_view) = factory.create_texture_const::<gfx::format::Rgba8>(
        gfx::tex::Kind::D2(1, 1, gfx::tex::AaMode::Single), &[&texels]
        ).unwrap();

    let sinfo = gfx::tex::SamplerInfo::new(
        gfx::tex::FilterMethod::Bilinear,
        gfx::tex::WrapMode::Clamp);

    let pso = factory.create_pipeline_simple(
        include_bytes!("shader/cube_150.glslv"),
        include_bytes!("shader/cube_150.glslf"),
        gfx::state::CullFace::Back,
        pipe::new()
    ).unwrap();

    let view: AffineMatrix3<f32> = Transform::look_at(
        Point3::new(1.5f32, -5.0, 3.0),
        Point3::new(0f32, 0.0, 0.0),
        Vector3::unit_z(),
    );
    let proj = cgmath::perspective(cgmath::deg(45.0f32), aspect_ratio, 1.0, 10.0);

    let data = pipe::Data {
        vbuf: vbuf,
        transform: (proj * view.mat).into(),
        locals: factory.create_constant_buffer(1),
        color: (texture_view, factory.create_sampler(sinfo)),
        out_color: main_color,
        out_depth: main_depth,
    };

    OpenGlRenderer {
      pso: pso,
      data: data,
      slice: slice,

      encoder: encoder,
      window: window,
      device: device,
    }
  }

  pub fn render_world(&mut self, _: &Option<&ClientWorld>) {
    // TODO: world
    self.render()
  }

  fn render(&mut self) {
    use cgmath::{Transform, AffineMatrix3};
    use cgmath::{Point3, Vector3};

    println!("render");
    let locals = Locals { transform: self.data.transform };
    self.encoder.update_constant_buffer(&self.data.locals, &locals);
    self.encoder.clear(&self.data.out_color, [0.1, 0.2, 0.3, 1.0]);
    self.encoder.clear_depth(&self.data.out_depth, 1.0);
    self.encoder.draw(&self.slice, &self.pso, &self.data);
    self.encoder.flush(&mut self.device);

    self.window.swap_buffers().unwrap();
    self.device.cleanup();
  }

}