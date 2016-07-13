use gfx_app::shade::Source;

pub struct Shader<'a> {
  pub vertex: Source<'a>,
  pub fragment: Source<'a>,
}

pub mod constants {
  use gfx_app;
  use super::Shader;

  pub fn cube_shader<'a>() -> Shader<'a> {
    let vs = gfx_app::shade::Source {
        glsl_120: include_bytes!("assets/shaders/cube_120.glslv"),
        glsl_150: include_bytes!("assets/shaders/cube_150.glslv"),
        .. gfx_app::shade::Source::empty()
    };
    let fs = gfx_app::shade::Source {
        glsl_120: include_bytes!("assets/shaders/cube_120.glslf"),
        glsl_150: include_bytes!("assets/shaders/cube_150.glslf"),
        .. gfx_app::shade::Source::empty()
    };

    Shader {
      vertex: vs,
      fragment: fs
    }
  }
}
