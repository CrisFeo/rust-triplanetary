use macroquad::prelude::{
  IVec2,
  draw_circle as mq_draw_circle,
  draw_line as mq_draw_line,
};
use crate::anchor::MIDDLE_CENTER;
use crate::color::Color;

pub fn draw_circle(color: Color, radius: i32, center: IVec2) {
  let offset = MIDDLE_CENTER.window_offset();
  let center = (center + offset).as_vec2();
  mq_draw_circle(center.x, center.y, radius as f32, color);
}

pub fn draw_line(color: Color, width: i32, a: IVec2 , b: IVec2) {
  let offset = MIDDLE_CENTER.window_offset();
  let a = (a + offset).as_vec2();
  let b = (b + offset).as_vec2();
  mq_draw_line(a.x, a.y, b.x, b.y, width as f32, color);
}
