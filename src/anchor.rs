use macroquad::prelude::{
  IVec2,
  ivec2,
  screen_width,
  screen_height,
};

#[derive(Copy, Clone, Debug)]
enum Horizontal {
  Left,
  Middle,
  Right,
}

#[derive(Copy, Clone, Debug)]
enum Vertical {
  Top,
  Center,
  Bottom,
}

#[derive(Copy, Clone, Debug)]
pub struct Anchor {
  h: Horizontal,
  v: Vertical,
}

pub const LEFT_TOP: Anchor =      Anchor { h: Horizontal::Left,   v: Vertical::Top    };
pub const MIDDLE_TOP: Anchor =    Anchor { h: Horizontal::Middle, v: Vertical::Top    };
pub const RIGHT_TOP: Anchor =     Anchor { h: Horizontal::Right,  v: Vertical::Top    };
pub const LEFT_CENTER: Anchor =   Anchor { h: Horizontal::Left,   v: Vertical::Center };
pub const MIDDLE_CENTER: Anchor = Anchor { h: Horizontal::Middle, v: Vertical::Center };
pub const RIGHT_CENTER: Anchor =  Anchor { h: Horizontal::Right,  v: Vertical::Center };
pub const LEFT_BOTTOM: Anchor =   Anchor { h: Horizontal::Left,   v: Vertical::Bottom };
pub const MIDDLE_BOTTOM: Anchor = Anchor { h: Horizontal::Middle, v: Vertical::Bottom };
pub const RIGHT_BOTTOM: Anchor =  Anchor { h: Horizontal::Right,  v: Vertical::Bottom };

impl Anchor {
  pub fn window_offset(self) -> IVec2 {
    self.offset(screen_width() as i32, screen_height() as i32)
  }

  pub fn offset(self, w: i32, h: i32) -> IVec2 {
    let x = match self.h {
      Horizontal::Left   => 0,
      Horizontal::Middle => w / 2,
      Horizontal::Right  => w,
    };
    let y = match self.v {
      Vertical::Top   => 0,
      Vertical::Center => h / 2,
      Vertical::Bottom  => h,
    };
    ivec2(x, y)
  }
}
