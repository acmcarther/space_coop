use automatic_system_installer::AutoInstaller;

use network;
use physics;
use player;
use state::Delta;

pub fn install_auto_systems(installer: &mut AutoInstaller<Delta>) {
  installer.auto_install::<network::DistributionSystem>();
  installer.auto_install::<player::ConnectionSystem>();
  installer.auto_install::<player::HealthCheckSystem>();
  installer.auto_install::<player::InputSystem>();
  installer.auto_install::<player::SnapshotSystem>();
  installer.auto_install::<physics::System>();
}
