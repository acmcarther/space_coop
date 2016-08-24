use automatic_system_installer::AutoInstaller;
use state::Delta;

use network;
use console;
use pause;
use debug;
use camera;
use synchronization;
use player;
use mutator;
use mouse_lock;

pub fn install_auto_systems(installer: &mut AutoInstaller<Delta>) {
  installer.auto_install::<network::EventDistributionSystem>();
  installer.auto_install::<network::ConnectionSystem>();
  installer.auto_install::<pause::System>();
  installer.auto_install::<player::PreprocessorSystem>();
  installer.auto_install::<mouse_lock::System>();
  installer.auto_install::<camera::PreprocessorSystem>();
  installer.auto_install::<console::PreprocessorSystem>();
  installer.auto_install::<player::MoveSystem>();
  installer.auto_install::<camera::MovementSystem>();
  installer.auto_install::<console::InputSystem>();
  installer.auto_install::<console::InvokeSystem>();
  installer.auto_install::<mutator::System>();
  installer.auto_install::<synchronization::System>();
  installer.auto_install::<network::KeepAliveSystem>();
  installer.auto_install::<debug::System>();
}
