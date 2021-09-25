use crate::entity::{
  ENTITY_MAX,
  EntityId,
};

pub struct Components<T> {
  by_id: [Option<T>; ENTITY_MAX],
}

impl<T> Components<T>{
  const NONE: Option<T> = None;

  pub fn new() -> Self {
    Components {
      by_id: [Self::NONE; ENTITY_MAX],
    }
  }

  pub fn has(&self, entity_id: EntityId) -> bool {
    match self.by_id[entity_id] {
      Some(_) => true,
      None => false,
    }
  }

  pub fn get(&self, entity_id: EntityId) -> &Option<T> {
    &self.by_id[entity_id]
  }

  pub fn get_mut(&mut self, entity_id: EntityId) -> &mut Option<T> {
    &mut self.by_id[entity_id]
  }

  pub fn set(&mut self, entity_id: EntityId, component: T) {
    self.by_id[entity_id] = Some(component);
  }

  pub fn del(&mut self, entity_id: EntityId) {
    self.by_id[entity_id] = None;
  }
}

