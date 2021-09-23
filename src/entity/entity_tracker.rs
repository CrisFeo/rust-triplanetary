use std::iter::Map;
use std::ops::Range;

use super::entity_id::{
  EntityId,
  new_entity_id,
};

pub const ENTITY_MAX: usize = 4096;

pub struct EntityTracker {
  next_index: u16,
}

impl EntityTracker {
  pub fn new() -> Self {
    EntityTracker { next_index: 0 }
  }

  pub fn create(&mut self) -> EntityId {
    let index = self.next_index;
    self.next_index += 1;
    new_entity_id(index)
  }

  pub fn all(&self) -> Map<Range<u16>, fn(u16) -> EntityId> {
    let count = u16::into(self.next_index);
    (0..count).map(new_entity_id as fn(u16) -> EntityId)
  }
}
