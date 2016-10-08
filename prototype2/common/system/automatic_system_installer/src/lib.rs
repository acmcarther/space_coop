extern crate itertools;
extern crate specs;
extern crate system_installer;
extern crate dag;

pub use dag::{Dag, PriorityMap};
use itertools::Itertools;
use std::any::{Any, TypeId};
use std::convert::From;
pub use system_installer::*;

/// A trait that systems who'd like to be automatically installable must
/// implement to tell the autoinstaller what they depend on. It is used to
/// define the run order of services within a tick.
///
/// A note: This advocates that systems should specify other systems as
/// dependencies rather than data. Though specifying data is more intuitive and
/// easier to maintain, it is less flexible. Several services may publish the
/// same data, or several services may consume the same data separately. In
/// both cases, we may not want to couple run order to data consumed.
///
/// One extreme case: the network adapter service has a data dependency on many
/// services that publish outgoing events, but it should not depend on them, or
/// we'd get a cycle.
pub trait DependencyAware {
  fn dependencies(&self) -> Vec<TypeId>;
  fn identity(&self) -> String;
}

/// A type that can install Installers and StandaloneInstallers
pub struct AutoInstaller<T: 'static> {
  install_thunks: Vec<Box<LazyInstallFn<T>>>,
  world: specs::World,
  dependency_set: Dag<TypeId>,
}

type LazyInstallFn<T> = FnMut(&mut specs::Planner<T>, &PriorityMap<TypeId>);

impl<T> AutoInstaller<T> {
  pub fn with_world(world: specs::World) -> AutoInstaller<T> {
    AutoInstaller {
      install_thunks: Vec::new(),
      world: world,
      dependency_set: Dag::new(),
    }
  }

  pub fn new() -> AutoInstaller<T> {
    AutoInstaller {
      install_thunks: Vec::new(),
      world: specs::World::new(),
      dependency_set: Dag::new(),
    }
  }

  pub fn take_dag(self) -> Dag<TypeId> {
    self.dependency_set
  }

  pub fn mut_world(&mut self) -> &mut specs::World {
    &mut self.world
  }

  pub fn auto_install<U: Any + StandaloneInstaller<T> + DependencyAware>
    (&mut self)
     -> &mut AutoInstaller<T> {
    let instance = U::from_world(&mut self.world);
    self.auto_install_instance(instance);
    self
  }

  pub fn auto_install_instance<U: Any + specs::System<T> + DependencyAware>
    (&mut self,
     installer: U)
     -> &mut AutoInstaller<T> {
    let own_type = TypeId::of::<U>();
    self.dependency_set.add_alias(&own_type, installer.identity());
    self.dependency_set.add_system(&own_type);
    self.dependency_set.add_dependency_set(&own_type, installer.dependencies().as_slice());
    let mut closure_own_type = Some(own_type);
    let mut closure_installer = Some(installer);

    self.install_thunks.push(Box::new(move |ref mut planner, ref priority_map| {
      // Bookkeeping to let this be a FnMut instead of a FnOnce
      // See: https://stackoverflow.com/questions/30411594/moving-a-boxed-function
      let installer = closure_installer.take().unwrap();
      let own_type = closure_own_type.take().unwrap();

      let name = format!("{:?}", own_type.clone());

      planner.install_instance(installer,
                               &name,
                               priority_map.get(&own_type).unwrap() as specs::Priority);
    }));
    self
  }

  pub fn apply(self, num_threads: usize) -> specs::Planner<T> {
    let priority_map = PriorityMap::from(self.dependency_set);
    let mut planner = specs::Planner::new(self.world, num_threads);
    self.install_thunks.into_iter().foreach(|mut thunk| thunk(&mut planner, &priority_map));

    planner
  }
}

#[macro_export]
macro_rules! declare_dependencies {
  ($thing:ty, [$($on:ty),*]) => {
    impl $crate::DependencyAware for $thing {
      fn dependencies(&self) -> Vec<::std::any::TypeId> {
        vec![$(::std::any::TypeId::of::<$on>()),*]
      }

      fn identity(&self) -> String {
        format!("{}::{}", module_path!(), stringify!($thing))
      }
    }
  }
}

#[macro_export]
macro_rules! standalone_installer_from_new {
  ($thing:ty, $delta:ty) => {
    impl $crate::StandaloneInstaller<$delta> for $thing {
      fn from_world(w: &mut ::specs::World) -> $thing {
        Self::new(w)
      }
    }
  }
}


#[cfg(test)]
mod tests {
  use specs;
  use std::any::TypeId;
  use super::*;

  macro_rules! null_system {
    ($thing:ty, $msg:expr) => {
      impl<T> specs::System<T> for $thing {
        fn run(&mut self, arg: specs::RunArg, _: T) {
          arg.fetch(|_| ());
          println!($msg);
        }
      }
    }
  }

  struct MockSystem1;
  null_system!(MockSystem1, "spin 1");
  declare_dependencies!(MockSystem1, []);

  impl<T: 'static> StandaloneInstaller<T> for MockSystem1 {
    fn from_world(_: &mut specs::World) -> MockSystem1 {
      MockSystem1
    }
  }

  struct MockSystem2;

  null_system!(MockSystem2, "spin 2");
  declare_dependencies!(MockSystem2, []);

  struct MockSystem3;
  null_system!(MockSystem3, "spin 3");
  declare_dependencies!(MockSystem3, [MockSystem2, MockSystem4]);

  struct MockSystem4;
  null_system!(MockSystem4, "spin 4");
  declare_dependencies!(MockSystem4, []);

  #[test]
  fn test() {
    let mut i = AutoInstaller::<()>::new();

    i.auto_install::<MockSystem1>();
    i.auto_install_instance(MockSystem2);
    i.auto_install_instance(MockSystem3);
    i.auto_install_instance(MockSystem4);

    let mut planner = i.apply(2);

    planner.dispatch(());
    // panic!("testing!")
  }
}
