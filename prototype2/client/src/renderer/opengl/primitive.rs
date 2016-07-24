use gfx::format;

pub type ColorFormat = format::Rgba8;

pub type DepthFormat = format::DepthStencil;

#[cfg_attr(rustfmt, rustfmt_skip)] // Macro syntax requires trailing comma
gfx_defines! {
  vertex Vertex {
    pos: [f32; 3] = "a_Pos",
    normal: [f32; 3] = "a_Norm",
    tex_coord: [f32; 2] = "a_TexCoord",
  }

  /*
  constant Locals {
    camera_pv: [[f32; 4]; 4] = "u_cameraPV",
    obj_to_world: [[f32; 4]; 4] = "u_objToWorld",
    norm_to_world: [[f32; 4]; 4] = "u_normToWorld",
    camera_pos: [f32; 3] = "u_cameraPos",
    light_pos: [f32; 3] = "u_lightPos",
  }
  */

  // pub mod pipe::{Data, Meta}
  pipeline pipe {
    vbuf: gfx::VertexBuffer<Vertex> = (),
    color: gfx::TextureSampler<[f32; 4]> = "t_Color",
    camera_pv: gfx::Global<[[f32; 4]; 4]> = "u_cameraPV",
    obj_to_world: gfx::Global<[[f32; 4]; 4]> = "u_objToWorld",
    norm_to_world: gfx::Global<[[f32; 4]; 4]> = "u_normToWorld",
    camera_pos: gfx::Global<[f32; 3]> = "u_cameraPos",
    light_pos: gfx::Global<[f32; 3]> = "u_lightPos",
    //locals: gfx::ConstantBuffer<Locals> = "Locals",
    out_color: gfx::RenderTarget<ColorFormat> = "o_color",
    out_depth: gfx::DepthTarget<DepthFormat> =
      gfx::preset::depth::LESS_EQUAL_WRITE,
  }
}

// Declare the vertex format suitable for drawing.
// Notice the use of FixedPoint.
impl Vertex {
  pub fn new(p: [f32; 3], n: [f32; 3], t: [f32; 2]) -> Vertex {
    Vertex {
      pos: [p[0], p[1], p[2]],
      normal: [n[0], n[1], n[2]],
      tex_coord: t,
    }
  }
}
