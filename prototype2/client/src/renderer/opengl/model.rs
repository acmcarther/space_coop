use renderer::opengl::primitive::Vertex;

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

pub mod constants {
  use super::Model;
  use renderer::opengl::primitive::Vertex;
  use common::world::Model as WorldModel;

  pub fn icosphere(iterations: u32) -> Model {
    let world_model = WorldModel::icosphere(iterations);

    Model {
      vertices: world_model.vertices.into_iter().map(|(v1, v2, v3)| {
        Vertex::new([v1, v2, v3], [v1, v2, v3], [1.0, 1.0])//[v1.sin() / f32::consts::PI + 0.5, v2.sin() / f32::consts::PI + 0.5])
      }).collect(),
      indices: world_model.indices
    }
  }

  pub fn cube() -> Model {
    let vertex_data = vec![
        // top (0.0, 0, 1.0)
        Vertex::new([-1.0, -1.0,  1.0], [0.0, 0.0, 1.0], [0.0, 0.0]),
        Vertex::new([ 1.0, -1.0,  1.0], [0.0, 0.0, 1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [0.0, 0.0, 1.0], [1.0, 1.0]),
        Vertex::new([-1.0,  1.0,  1.0], [0.0, 0.0, 1.0], [0.0, 1.0]),
        // bottom (0.0, 0, -1.0)
        Vertex::new([-1.0,  1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0, -1.0], [0.0, 0.0, -1.0], [0.0, 0.0]),
        Vertex::new([ 1.0, -1.0, -1.0], [0.0, 0.0, -1.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, 0.0, -1.0], [1.0, 1.0]),
        // right (1.0, 0.0, 0)
        Vertex::new([ 1.0, -1.0, -1.0], [1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([ 1.0,  1.0, -1.0], [1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [1.0, 0.0, 0.0], [1.0, 1.0]),
        Vertex::new([ 1.0, -1.0,  1.0], [1.0, 0.0, 0.0], [0.0, 1.0]),
        // left (-1.0, 0.0, 0)
        Vertex::new([-1.0, -1.0,  1.0], [-1.0, 0.0, 0.0], [1.0, 0.0]),
        Vertex::new([-1.0,  1.0,  1.0], [-1.0, 0.0, 0.0], [0.0, 0.0]),
        Vertex::new([-1.0,  1.0, -1.0], [-1.0, 0.0, 0.0], [0.0, 1.0]),
        Vertex::new([-1.0, -1.0, -1.0], [-1.0, 0.0, 0.0], [1.0, 1.0]),
        // front (0.0, 1.0, 0)
        Vertex::new([ 1.0,  1.0, -1.0], [0.0, 1.0, 0.0], [1.0, 0.0]),
        Vertex::new([-1.0,  1.0, -1.0], [0.0, 1.0, 0.0], [0.0, 0.0]),
        Vertex::new([-1.0,  1.0,  1.0], [0.0, 1.0, 0.0], [0.0, 1.0]),
        Vertex::new([ 1.0,  1.0,  1.0], [0.0, 1.0, 0.0], [1.0, 1.0]),
        // back (0.0, -1.0, 0)
        Vertex::new([ 1.0, -1.0,  1.0], [0.0, -1.0, 0.0], [0.0, 0.0]),
        Vertex::new([-1.0, -1.0,  1.0], [0.0, -1.0, 0.0], [1.0, 0.0]),
        Vertex::new([-1.0, -1.0, -1.0], [0.0, -1.0, 0.0], [1.0, 1.0]),
        Vertex::new([ 1.0, -1.0, -1.0], [0.0, -1.0, 0.0], [0.0, 1.0]),
    ];

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
