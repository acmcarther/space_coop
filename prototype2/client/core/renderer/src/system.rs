use camera;
use common::aspects::{DisabledAspect, PhysicalAspect, RenderAspect, SynchronizedAspect};
use console;
use debug;
use gfx;
use gfx::Device;
use gfx_device_gl;
use gfx_device_gl::{CommandBuffer, Resources};
use glutin;
use opengl::OpenGlRenderer;
use pause;
use specs;
use state::OwnEntity;
use std::ops::Not;
use std::sync::RwLockReadGuard;

// NOTE: This isn't a "real" system. It's not Send, so it has to be invoked in
// the main thread. It is expected to be run at the end of the tick, but thats
// not really a hard requirement.
pub struct System {
  renderer: OpenGlRenderer,
}

pub type Encoder = gfx::Encoder<Resources, CommandBuffer>;

impl System {
  pub fn new(renderer: OpenGlRenderer) -> System {
    System { renderer: renderer }
  }

  pub fn run(&mut self,
             w: &mut specs::World,
             encoder: &mut Encoder,
             device: &mut gfx_device_gl::Device) {
    let (mut window,
         camera_pos,
         own_entity,
         debug_msg,
         pause_state,
         command_buffer,
         console_log,
         synchronized,
         entities,
         physical,
         disabled,
         render) = (w.write_resource::<glutin::Window>(),
                    w.read_resource::<camera::CameraPos>(),
                    w.write_resource::<Option<OwnEntity>>(),
                    w.read_resource::<debug::DebugMessage>(),
                    w.read_resource::<pause::PauseState>(),
                    w.read_resource::<console::CommandBuffer>(),
                    w.read_resource::<console::ConsoleLog>(),
                    w.read::<SynchronizedAspect>(),
                    w.entities(),
                    w.read::<PhysicalAspect>(),
                    w.read::<DisabledAspect>(),
                    w.read::<RenderAspect>());

    let camera_target = SelfRetriever::new(own_entity.clone(), &entities, &synchronized, &physical)
      .find_own_pos();

    RenderWrapper::new(&mut self.renderer,
                       encoder,
                       &mut window,
                       &camera_pos,
                       camera_target,
                       &debug_msg,
                       &pause_state,
                       &command_buffer,
                       &console_log,
                       &render,
                       &physical,
                       &disabled)
      .render_frame();

    encoder.flush(device);
    window.swap_buffers().unwrap();
    device.cleanup();
  }
}

// TODO(acmcarther): Move the below (all of it) somewhere else. This is here
// from the OpenGl service refactoring
type AspectStorageRead<'a, T> = specs::Storage<T,
                                               RwLockReadGuard<'a, specs::Allocator>,
                                               RwLockReadGuard<'a, specs::MaskedStorage<T>>>;

// TODO: Document
struct SelfRetriever<'a> {
  own_entity: Option<OwnEntity>,
  entities: &'a specs::Entities<'a>,
  synchronized: &'a AspectStorageRead<'a, SynchronizedAspect>,
  physical: &'a AspectStorageRead<'a, PhysicalAspect>,
}

impl<'a> SelfRetriever<'a> {
  pub fn new(own_entity: Option<OwnEntity>,
             entities: &'a specs::Entities<'a>,
             synchronized: &'a AspectStorageRead<'a, SynchronizedAspect>,
             physical: &'a AspectStorageRead<'a, PhysicalAspect>)
             -> SelfRetriever<'a> {

    SelfRetriever {
      own_entity: own_entity,
      entities: entities,
      synchronized: synchronized,
      physical: physical,
    }
  }

  pub fn find_own_pos(mut self) -> Option<(f32, f32, f32)> {
    use specs::Join;

    self.own_entity
      .take()
      .and_then(|ent| {
        (self.entities, self.synchronized)
          .iter()
          .filter(|&(_, synchro)| {
            let OwnEntity(ref own_ent) = ent;
            synchro == own_ent
          })
          .next()
      })
      .map(|(entity, _)| entity)
      .and_then(|true_ent| self.physical.get(true_ent))
      .map(|physical_aspect| physical_aspect.pos)
  }
}

// TODO(acmcarther): Document and rename to something more descriptive
// TODO(acmcarther): This seems like it defeats the purpose: it takes *way* too
// many params
struct RenderWrapper<'a> {
  renderer: &'a mut OpenGlRenderer,
  encoder: &'a mut gfx::Encoder<Resources, CommandBuffer>,
  window: &'a mut glutin::Window,
  camera_pos: &'a camera::CameraPos,
  camera_target: Option<(f32, f32, f32)>,
  debug_msg: &'a debug::DebugMessage,
  pause_state: &'a pause::PauseState,
  command_buffer: &'a console::CommandBuffer,
  console_log: &'a console::ConsoleLog,
  render: &'a AspectStorageRead<'a, RenderAspect>,
  physical: &'a AspectStorageRead<'a, PhysicalAspect>,
  disabled: &'a AspectStorageRead<'a, DisabledAspect>,
}

impl<'a> RenderWrapper<'a> {
  pub fn new(renderer: &'a mut OpenGlRenderer,
             encoder: &'a mut gfx::Encoder<Resources, CommandBuffer>,
             window: &'a mut glutin::Window,
             camera_pos: &'a camera::CameraPos,
             camera_target: Option<(f32, f32, f32)>,
             debug_msg: &'a debug::DebugMessage,
             pause_state: &'a pause::PauseState,
             command_buffer: &'a console::CommandBuffer,
             console_log: &'a console::ConsoleLog,
             render: &'a AspectStorageRead<'a, RenderAspect>,
             physical: &'a AspectStorageRead<'a, PhysicalAspect>,
             disabled: &'a AspectStorageRead<'a, DisabledAspect>)
             -> RenderWrapper<'a> {
    RenderWrapper {
      renderer: renderer,
      encoder: encoder,
      window: window,
      camera_pos: camera_pos,
      camera_target: camera_target,
      debug_msg: debug_msg,
      pause_state: pause_state,
      command_buffer: command_buffer,
      console_log: console_log,
      render: render,
      physical: physical,
      disabled: disabled,
    }
  }

  pub fn render_frame(mut self) {
    use specs::Join;
    use itertools::Itertools;

    let &camera::CameraPos(x, y, z) = self.camera_pos;

    // TODO: clean up this api: we shouldnt rely on implict state setting here
    self.renderer.render_world(&mut self.encoder,
                               &mut self.window,
                               &(x, y, z),
                               self.camera_target);

    (self.physical, self.disabled.not(), self.render)
      .iter()
      .foreach(|(physical_aspect, _, render_aspect)| {
        self.renderer.render_model(&mut self.encoder,
                                   &mut self.window,
                                   physical_aspect,
                                   render_aspect,
                                   (x, y, z));
      });

    self.renderer.render_ui(&mut self.encoder,
                            &mut self.window,
                            &self.debug_msg,
                            &self.command_buffer,
                            &self.console_log,
                            &self.pause_state);
  }
}
