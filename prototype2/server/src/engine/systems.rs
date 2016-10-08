use automatic_system_installer::AutoInstaller;
use state::Delta;

use network;
use player;
use physics;

pub fn install_auto_systems(installer: &mut AutoInstaller<Delta>) {
  installer.auto_install::<network::DistributionSystem>();
  installer.auto_install::<player::ConnectionSystem>();
  installer.auto_install::<player::HealthCheckSystem>();
  installer.auto_install::<player::InputSystem>();
  installer.auto_install::<player::SnapshotSystem>();
  installer.auto_install::<physics::System>();
}
