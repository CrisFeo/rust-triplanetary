use macroquad::prelude::{
  Color,
  IVec2,
};
use crate::anchor::{
  MIDDLE_CENTER,
  Anchor,
};
use super::sprite::{
  Sprite,
};

struct Draw<'a>(&'a Sprite, Color, Anchor, IVec2);

pub struct SpriteLayer<'a> {
  draws: Vec<Draw<'a>>,
}

impl<'a> SpriteLayer<'a> {
  pub fn new() -> SpriteLayer<'static> {
    SpriteLayer {
      draws: vec![],
    }
  }

  pub fn begin(&mut self) {
    self.draws.clear();
  }

  pub fn draw(&mut self, sprite: &'a Sprite, color: Color, position: IVec2) {
    self.draw_with_anchor(sprite, color, MIDDLE_CENTER, position);
  }

  pub fn draw_with_anchor(&mut self, sprite: &'a Sprite, color: Color, anchor: Anchor, position: IVec2) {
    self.draws.push(Draw(sprite, color, anchor, position));
  }

  pub fn end(&mut self) {
    self.draws.sort_by(|Draw(_, _, _, a), Draw(_, _, _, b)| a.y.cmp(&b.y));
    for Draw(sprite, color, anchor, position) in &self.draws {
      sprite.draw_with_anchor(*color, *anchor, *position);
    }
  }
}

