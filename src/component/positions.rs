use std::collections::{
  HashMap,
  HashSet,
};
use crate::entity::EntityId;
use crate::hex::Hex;
use super::components::Components;

#[derive(Default)]
pub struct Positions {
  components: Components<Hex>,
  by_hex: HashMap<Hex, HashSet<EntityId>>,
}

impl Positions {
  pub fn at(&self, hex: Hex) -> Option<&HashSet<EntityId>> {
    self.by_hex.get(&hex)
  }

  pub fn has(&self, entity_id: EntityId) -> bool {
    self.components.has(entity_id)
  }

  pub fn get(&self, entity_id: EntityId) -> Option<&Hex> {
    self.components.get(entity_id)
  }

  pub fn set(&mut self, entity_id: EntityId, hex: Hex) {
    if let Some(previous) = self.components.get(entity_id) {
      self.by_hex.entry(*previous).and_modify(|v| { v.remove(&entity_id); });
    }
    self.components.set(entity_id, hex);
    self.by_hex.entry(hex)
      .and_modify(|v| { v.insert(entity_id); })
      .or_insert_with(|| {
        let mut hs = HashSet::with_capacity(1);
        hs.insert(entity_id);
        hs
      });
  }

  pub fn del(&mut self, entity_id: EntityId) {
    if let Some(current) = self.components.get(entity_id) {
      self.by_hex.entry(*current).and_modify(|v| { v.remove(&entity_id); });
    }
    self.components.del(entity_id);
  }
}

