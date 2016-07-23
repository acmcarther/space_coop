pub struct Shader {
  glsl_150v: Vec<u8>,
  glsl_150f: Vec<u8>,
}

impl Shader {
  pub fn get_vertex(&self) -> &[u8] {
    &self.glsl_150v
  }

  pub fn get_fragment(&self) -> &[u8] {
    &self.glsl_150f
  }
}

pub mod constants {
  use super::*;
  use std::fs::File;
  use std::io::Read;

  pub fn cube_shader() -> Shader {
    let glsl_150v: Vec<u8> = File::open("../assets/shaders/cube_150.glslv").unwrap().bytes().map(|x| x.unwrap()).collect();
    let glsl_150f: Vec<u8> = File::open("../assets/shaders/cube_150.glslf").unwrap().bytes().map(|x| x.unwrap()).collect();

    assert!(!glsl_150v.is_empty());
    assert!(!glsl_150f.is_empty());

    Shader {
      glsl_150v: glsl_150v,
      glsl_150f: glsl_150f,
    }
  }
}
