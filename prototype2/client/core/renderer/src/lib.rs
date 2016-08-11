extern crate time;
extern crate specs;
extern crate serde;
extern crate serde_json;
extern crate itertools;
extern crate cgmath;
#[macro_use]
extern crate gfx;
extern crate glutin;
extern crate gfx_text;
extern crate gfx_window_glutin;
extern crate gfx_device_gl;

extern crate common;
extern crate console;
extern crate pause;
extern crate debug;
extern crate camera;

pub mod opengl;
pub mod system;

pub use system::System as RenderingSystem;
