pub use gfx_app::{ColorFormat, DepthFormat};

// Declare the vertex format suitable for drawing.
// Notice the use of FixedPoint.
gfx_vertex_struct!( Vertex {
  pos: [i8; 4] = "a_Pos",
  tex_coord: [i8; 2] = "a_TexCoord",
});

impl Vertex {
  pub fn new(p: [i8; 3], t: [i8; 2]) -> Vertex {
    Vertex {
      pos: [p[0], p[1], p[2], 1],
      tex_coord: t,
    }
  }
}

// pub struct Locals
gfx_constant_struct!( Locals {
  transform: [[f32; 4]; 4] = "u_Transform",
});

// pub mod pipe::{Data, Meta}
gfx_pipeline!( pipe {
  vbuf: gfx::VertexBuffer<Vertex> = (),
  transform: gfx::Global<[[f32; 4]; 4]> = "u_Transform",
  locals: gfx::ConstantBuffer<Locals> = "Locals",
  color: gfx::TextureSampler<[f32; 4]> = "t_Color",
  out_color: gfx::RenderTarget<ColorFormat> = "Target0",
  out_depth: gfx::DepthTarget<DepthFormat> =
    gfx::preset::depth::LESS_EQUAL_WRITE,
});

