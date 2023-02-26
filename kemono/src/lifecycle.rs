use fxhash::FxHashMap;
use legion::{Resources, Schedule, World};
use legion::systems::Builder;
use strum::IntoEnumIterator;
use strum_macros::EnumIter;

#[derive(Copy, Clone, PartialEq, Eq, Hash, EnumIter)]
pub enum Event {
  PreStart,
  Start,
  Tick,
  Update,
  PostUpdate,
  Stop
}

pub struct Lifecycle {
  schedules: FxHashMap<Event, Schedule>
}

impl Lifecycle {
  pub fn run_systems(&mut self, event: Event, world: &mut World, resources: &mut Resources) {
    self.schedules.get_mut(&event).unwrap().execute(world, resources);
  }
}

pub struct LifecycleBuilder {
  pub builders: FxHashMap<Event, Builder>
}

impl LifecycleBuilder {
  pub fn new() -> Self {
    let mut builders: FxHashMap<Event, Builder> = FxHashMap::default();

    for event in Event::iter() {
      builders.insert(event, Schedule::builder());
    }

    Self {
      builders,
    }
  }

  pub fn build(mut self) -> Lifecycle {
    let mut schedules: FxHashMap<Event, Schedule> = FxHashMap::default();

    for event in Event::iter() {
      schedules.insert(event, self.builders.get_mut(&event).unwrap().build());
    }

    Lifecycle {
      schedules,
    }
  }
}

impl Default for LifecycleBuilder {
  fn default() -> Self {
    Self::new()
  }
}