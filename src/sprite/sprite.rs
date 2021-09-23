use macroquad::prelude::*;

use super::anchor::{
  Anchor,
  anchor_offset,
};

pub struct Sprite {
  pub pivot_x: i32,
  pub pivot_y: i32,
  texture: Texture2D,
  width: f32,
  height: f32,
  source: Rect,
}

impl Sprite {
  pub fn new(texture: Texture2D, pivot: Anchor, scale: i32, w: i32, h: i32, x: i32, y: i32) -> Sprite {
    let (pivot_x, pivot_y) = anchor_offset(pivot, -w * scale, -h * scale);
    Sprite {
      pivot_x,
      pivot_y,
      texture,
      width: (w * scale) as f32,
      height: (h * scale) as f32,
      source: Rect::new(x as f32, y as f32, w as f32, h as f32)
    }
  }

  pub fn draw(&self, color: Color, anchor: Anchor, p: IVec2) {
    let (screen_x, screen_y) = {
      let screen_width = screen_width() as i32;
      let screen_height = screen_height() as i32;
      anchor_offset(anchor, screen_width, screen_height)
    };
    let x = screen_x + p.x + self.pivot_x;
    let y = screen_y + p.y + self.pivot_y;
    draw_texture_ex(
      self.texture,
      x as f32,
      y as f32,
      color,
      DrawTextureParams {
        dest_size: Some(Vec2::new(self.width, self.height)),
        source: Some(self.source),
        ..Default::default()
      },
    );
  }}

