use std::iter::Map;
use std::ops::Range;
use std::collections::VecDeque;
use super::entity_id::{
  EntityId,
  new_entity_id,
  index_of_entity_id,
};

pub const ENTITY_MAX: usize = 8192;
pub const REUSE_SIZE: usize = 100;

pub struct EntityTracker {
  next_index: u16,
  unused_indices: VecDeque<u16>,
}

impl EntityTracker {
  pub fn new() -> Self {
    EntityTracker {
      next_index: 0,
      unused_indices: VecDeque::new(),
    }
  }

  pub fn create(&mut self) -> EntityId {
    let index = {
      if self.unused_indices.len() > REUSE_SIZE {
        self.unused_indices.pop_front().unwrap()
      } else {
        let index = self.next_index;
        self.next_index += 1;
        index
      }
    };
    new_entity_id(index)
  }

  pub fn remove(&mut self, entity_id: EntityId) {
    self.unused_indices.push_back(index_of_entity_id(entity_id));
  }

  pub fn all(&self) -> Map<Range<u16>, fn(u16) -> EntityId> {
    let count = u16::into(self.next_index);
    (0..count).map(new_entity_id as fn(u16) -> EntityId)
  }
}
