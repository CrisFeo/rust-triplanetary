use macroquad::prelude::{
  IVec2,
  Vec2,
  Texture2D,
  Rect,
  screen_width,
  screen_height,
  draw_texture_ex,
  DrawTextureParams,
};
use crate::anchor::{
  MIDDLE_CENTER,
  Anchor,
};
use crate::color::Color;

pub struct Sprite {
  pub pivot_x: i32,
  pub pivot_y: i32,
  texture: Texture2D,
  width: f32,
  height: f32,
  source: Rect,
}

impl Sprite {
  pub fn new(texture: Texture2D, pivot: Anchor, w: i32, h: i32, x: i32, y: i32) -> Sprite {
    let pivot_position = pivot.offset(-w, -h);
    Sprite {
      pivot_x: pivot_position.x,
      pivot_y: pivot_position.y,
      texture,
      width: w as f32,
      height: h as f32,
      source: Rect::new(x as f32, y as f32, w as f32, h as f32)
    }
  }

  fn inside_screen(&self, x: i32, y: i32) -> bool {
    let screen_width = screen_width() as i32;
    let screen_height = screen_height() as i32;
    let within_x_upper = x < screen_width;
    let within_x_lower = x + self.width as i32 >= 0;
    let within_y_upper = y < screen_height;
    let within_y_lower = y + self.height as i32 >= 0;
    within_x_upper && within_x_lower && within_y_upper && within_y_lower
  }

  pub fn draw(&self, color: Color, p: IVec2) {
    self.draw_with_anchor(color, MIDDLE_CENTER, p);
  }

  pub fn draw_with_anchor(&self, color: Color, anchor: Anchor, p: IVec2) {
    let screen_position = {
      let screen_width = screen_width() as i32;
      let screen_height = screen_height() as i32;
      anchor.offset(screen_width, screen_height)
    };
    let x = screen_position.x + p.x + self.pivot_x;
    let y = screen_position.y + p.y + self.pivot_y;
    if self.inside_screen(x, y) {
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
    }
  }}

