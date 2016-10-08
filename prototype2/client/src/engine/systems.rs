use automatic_system_installer::AutoInstaller;
use camera;
use console;
use debug;
use mouse_lock;
use mutator;
use network;
use pause;
use player;
use state::Delta;
use synchronization;

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
