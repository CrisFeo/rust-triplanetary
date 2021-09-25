use core::ops::{
  Index,
  IndexMut,
};

#[derive(Copy, Clone, Debug, Hash)]
pub struct EntityId {
  index: u16,
}

pub fn new_entity_id(index: u16) -> EntityId {
  EntityId { index }
}

pub fn index_of_entity_id(entity_id: EntityId) -> u16 {
  entity_id.index
}

impl std::fmt::Display for EntityId {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "eid-{}", self.index)
  }
}

impl PartialEq for EntityId {
  fn eq(&self, other: &Self) -> bool {
    self.index == other.index
  }
}

impl Eq for EntityId {}

impl<T> Index<EntityId> for [T]
where
  [T]: Index<usize>,
{
  type Output = <[T] as Index<usize>>::Output;

  fn index(&self, entity_id: EntityId) -> &Self::Output {
    <[T] as Index<usize>>::index(self, entity_id.index as usize)
  }
}

impl<T> IndexMut<EntityId> for [T]
where
  [T]: IndexMut<usize>,
{
  fn index_mut(&mut self, entity_id: EntityId) -> &mut Self::Output {
    <[T] as IndexMut<usize>>::index_mut(self, entity_id.index as usize)
  }
}
