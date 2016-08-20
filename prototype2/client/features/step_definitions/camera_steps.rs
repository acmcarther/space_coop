use cucumber::CucumberRegistrar;

use support::{ClientWorld, window};
use std::str::FromStr;
use pubsub::PubSubStore;
use glutin::{Event, VirtualKeyCode, Window};
use camera::CameraPos;
use std::ops::Deref;

pub fn register_steps(c: &mut CucumberRegistrar<ClientWorld>) {
  When!(c,
        "^the camera is at (\\d+), (\\d+), (\\d+)$",
        |_, world: &mut ClientWorld, (d1, d2, d3): (i32, i32, i32)| {
          let mut pos = world.planner.mut_world().write_resource::<CameraPos>();
          *pos = CameraPos(d1 as f32, d2 as f32, d3 as f32);
        });

  Then!(c,
        "^the camera pos is the default pos$",
        |_, world: &mut ClientWorld, _| {
          let pos = world.planner.mut_world().read_resource::<CameraPos>();
          assert_eq!(pos.deref(), &CameraPos(3.0, -10.0, 6.0));
        });

  Then!(c,
        "^the camera moves left$",
        |_, world: &mut ClientWorld, _| {
          let &CameraPos(x, y, z) = world.planner.mut_world().read_resource::<CameraPos>().deref();
          assert!(x < 3.0, "X should have gone down");
          assert!(y < -10.0, "Y should have gone down");
        });

  Then!(c,
        "^the camera moves right$",
        |_, world: &mut ClientWorld, _| {
          let &CameraPos(x, y, z) = world.planner.mut_world().read_resource::<CameraPos>().deref();
          assert!(x > 3.0, "X should have gone up");
          assert!(y > -10.0, "Y should have gone up");
        });

  Then!(c,
        "^the camera target distance invariant holds$",
        |_, world: &mut ClientWorld, _| {
          let &CameraPos(x, y, z) = world.planner.mut_world().read_resource::<CameraPos>().deref();
          assert_eq!(145.0 /* dist_sq to target */, x * x + y * y + z * z)
        });
}
