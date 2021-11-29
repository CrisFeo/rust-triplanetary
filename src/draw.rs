use macroquad::prelude::{
  IVec2,
  draw_line as mq_draw_line,
};
use crate::anchor::MIDDLE_CENTER;
use crate::color::Color;

pub fn draw_line(color: Color, width: i32, a: IVec2 , b: IVec2) {
  let offset = MIDDLE_CENTER.window_offset();
  let a = (a + offset).as_f32();
  let b = (b + offset).as_f32();
  mq_draw_line(a.x, a.y, b.x, b.y, width as f32, color);
}
