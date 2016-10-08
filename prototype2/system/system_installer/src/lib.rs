extern crate itertools;
extern crate specs;

/// A system that can be instantiated and installed by the InstallPlugin
pub trait StandaloneInstaller<T: 'static>: specs::System<T> {
  fn from_world(&mut specs::World) -> Self;
}

/// A type that can install Installers and StandaloneInstallers
pub trait InstallPlugin<T: 'static> {
  fn install<U: StandaloneInstaller<T>>(&mut self, name: &str, priority: specs::Priority);
  fn install_instance<U: specs::System<T>>(&mut self,
                                           installer: U,
                                           name: &str,
                                           priority: specs::Priority);
}

/// Implemented by default for specs::Planner
impl<T: 'static> InstallPlugin<T> for specs::Planner<T> {
  fn install<U: StandaloneInstaller<T> + 'static>(&mut self,
                                                  name: &str,
                                                  priority: specs::Priority) {
    let i = U::from_world(&mut self.mut_world());
    self.install_instance(i, name, priority);
  }

  fn install_instance<U: specs::System<T> + 'static>(&mut self,
                                                     instance: U,
                                                     name: &str,
                                                     priority: specs::Priority) {
    self.add_system(instance, name, priority);
  }
}
