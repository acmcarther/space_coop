use cgmath::{Matrix4, SquareMatrix};
use gfx;
use gfx::Factory;
use gfx::handle::{DepthStencilView, RenderTargetView};
use gfx::handle::ShaderResourceView;
use gfx::traits::FactoryExt;
use gfx_device_gl;
use opengl::primitive3d::{self, Vertex};
use opengl::primitive3d::{ColorFormat, DepthFormat /* Locals */};

#[derive(Debug)]
pub struct Model {
  pub vertices: Vec<Vertex>,
  pub indices: Vec<u16>,
}

impl Model {
  pub fn new(v: Vec<Vertex>, i: Vec<u16>) -> Model {
    Model {
      vertices: v,
      indices: i,
    }
  }
}

pub fn build_cube
  (factory: &mut gfx_device_gl::Factory,
   texture_view: ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
   color_out: RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
   depth_out: DepthStencilView<gfx_device_gl::Resources, DepthFormat>)
   -> (gfx::Slice<gfx_device_gl::Resources>, primitive3d::pipe::Data<gfx_device_gl::Resources>) {
  let cube_model = constants::cube();
  let (cube_vbuf, cube_slice) =
    factory.create_vertex_buffer_with_slice(cube_model.vertices.as_slice(),
                                            cube_model.indices.as_slice());

  let sinfo = gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Bilinear,
                                         gfx::tex::WrapMode::Clamp);

  (cube_slice,
   primitive3d::pipe::Data {
    vbuf: cube_vbuf,
    color: (texture_view, factory.create_sampler(sinfo)), // Thrown away
    // locals: factory.create_constant_buffer(1),
    camera_pv: Matrix4::identity().into(),
    obj_to_world: Matrix4::identity().into(),
    norm_to_world: Matrix4::identity().into(),
    camera_pos: [0.0, 0.0, 0.0],
    light_pos: [0.0, 0.0, 0.0, 0.0],
    out_color: color_out,
    out_depth: depth_out,
  })
}

pub fn build_icosphere
  (iterations: u32,
   factory: &mut gfx_device_gl::Factory,
   texture_view: ShaderResourceView<gfx_device_gl::Resources, [f32; 4]>,
   color_out: RenderTargetView<gfx_device_gl::Resources, ColorFormat>,
   depth_out: DepthStencilView<gfx_device_gl::Resources, DepthFormat>)
   -> (gfx::Slice<gfx_device_gl::Resources>, primitive3d::pipe::Data<gfx_device_gl::Resources>) {
  let cube_model = constants::icosphere(iterations);
  let (cube_vbuf, cube_slice) =
    factory.create_vertex_buffer_with_slice(cube_model.vertices.as_slice(),
                                            cube_model.indices.as_slice());

  let sinfo = gfx::tex::SamplerInfo::new(gfx::tex::FilterMethod::Bilinear,
                                         gfx::tex::WrapMode::Clamp);

  (cube_slice,
   primitive3d::pipe::Data {
    vbuf: cube_vbuf,
    color: (texture_view, factory.create_sampler(sinfo)),
    // locals: factory.create_constant_buffer(1),
    camera_pv: Matrix4::identity().into(),
    obj_to_world: Matrix4::identity().into(),
    norm_to_world: Matrix4::identity().into(),
    camera_pos: [0.0, 0.0, 0.0],
    light_pos: [0.0, 0.0, 0.0, 0.0],
    out_color: color_out,
    out_depth: depth_out,
  })
}

pub mod constants {
  use common::geometry::model::Model as WorldModel;
  use opengl::primitive3d::Vertex;
  use super::Model;

  pub fn icosphere(iterations: u32) -> Model {
    let world_model = WorldModel::icosphere(iterations);

    Model {
      vertices: world_model.vertices
        .into_iter()
        .map(|vertex| Vertex::new(vertex.pos, vertex.norm, vertex.uv))
        .collect(),
      indices: world_model.indices,
    }
  }

  pub fn cube() -> Model {
    let generic_cube = WorldModel::cube();
    let vertex_data = generic_cube.vertices
      .into_iter()
      .map(|vertex| Vertex::new(vertex.pos, vertex.norm, vertex.uv))
      .collect();

    let index_data = vec![
         0,  1,  2,  2,  3,  0, // top
         4,  5,  6,  6,  7,  4, // bottom
         8,  9, 10, 10, 11,  8, // right
        12, 13, 14, 14, 15, 12, // left
        16, 17, 18, 18, 19, 16, // front
        20, 21, 22, 22, 23, 20, // back
    ];

    Model::new(vertex_data, index_data)
  }
}
