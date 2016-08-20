#![feature(slice_patterns)]

extern crate itertools;
extern crate specs;
extern crate cgmath;
extern crate uuid;
extern crate glutin;
#[macro_use(lazy_static, __lazy_static_create)]
extern crate lazy_static;
extern crate pubsub;
extern crate pause;
extern crate client_state as state;
extern crate common;
#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

mod invoke;
mod charsets;
mod input;
mod preprocessor;
mod primitive_interpreter;

pub use self::invoke::ConsoleLog;
pub use self::invoke::System as InvokeSystem;
pub use self::input::{CommandBuffer, CommandCursor, ExecutedCommand};
pub use self::input::System as InputSystem;
pub use self::preprocessor::System as PreprocessorSystem;
pub use self::primitive_interpreter::Command;
