use std::collections::HashMap;
use crate::entity::EntityId;

pub struct Components<T> {
  by_id: HashMap<EntityId, T>,
}

impl<T> Default for Components<T> {
  fn default() -> Self {
    Self {
      by_id: HashMap::new(),
    }
  }
}

impl<T> Components<T>{
  pub fn has(&self, entity_id: EntityId) -> bool {
    self.by_id.contains_key(&entity_id)
  }

  pub fn get(&self, entity_id: EntityId) -> Option<&T> {
    self.by_id.get(&entity_id)
  }

  pub fn get_mut(&mut self, entity_id: EntityId) -> Option<&mut T> {
    self.by_id.get_mut(&entity_id)
  }

  pub fn set(&mut self, entity_id: EntityId, component: T) {
    self.by_id.insert(entity_id, component);
  }

  pub fn del(&mut self, entity_id: EntityId) {
    self.by_id.remove(&entity_id);
  }
}

