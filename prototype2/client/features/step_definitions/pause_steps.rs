use cucumber::CucumberRegistrar;
use support::ClientWorld;
use pause::PauseState;
use std::ops::Deref;

pub fn register_steps(c: &mut CucumberRegistrar<ClientWorld>) {
  When!(c,
        "^the game is set as paused$",
        |_, world: &mut ClientWorld, _| {
          let mut pause_state = world.planner.mut_world().write_resource::<PauseState>();
          *pause_state = PauseState::Paused;
        });

  When!(c,
        "^the game is set as unpaused$",
        |_, world: &mut ClientWorld, _| {
          let mut pause_state = world.planner.mut_world().write_resource::<PauseState>();
          *pause_state = PauseState::NotPaused;
        });

  Then!(c, "^the game is paused$", |_, world: &mut ClientWorld, _| {
    let mut pause_state = world.planner.mut_world().read_resource::<PauseState>();
    assert_eq!(pause_state.deref(), &PauseState::Paused)
  });

  Then!(c,
        "^the game is unpaused$",
        |_, world: &mut ClientWorld, _| {
          let pause_state = world.planner.mut_world().read_resource::<PauseState>();
          assert_eq!(pause_state.deref(), &PauseState::NotPaused)
        });
}
