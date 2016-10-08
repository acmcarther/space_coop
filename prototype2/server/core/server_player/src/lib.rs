extern crate specs;
extern crate time;
extern crate itertools;

extern crate common;
extern crate aspects;
extern crate server_network as network;
extern crate server_state as state;
extern crate physics;
extern crate pubsub;

#[macro_use(declare_dependencies, standalone_installer_from_new)]
extern crate automatic_system_installer;

mod connection;
mod health_check;
mod input;
mod snapshot;

pub use snapshot::System as SnapshotSystem;
pub use input::System as InputSystem;
pub use health_check::System as HealthCheckSystem;
pub use connection::System as ConnectionSystem;
