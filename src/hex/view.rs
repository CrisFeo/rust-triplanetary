use macroquad::math::{
  IVec2,
  ivec2,
};

use super::hex::{
  Hex,
};

pub enum HexOrientation {
  Flat,
  Pointy,
}

pub struct HexView {
  pub layout: HexOrientation,
  pub tile_width: i32,
  pub tile_height: i32,
}

impl HexView {
  pub fn new(layout: HexOrientation, tile_width: i32, tile_height: i32) -> HexView {
    HexView {
      layout,
      tile_width,
      tile_height,
    }
  }

  pub fn to_pixel(&self, hex: Hex) -> IVec2 {
    match self.layout {
      HexOrientation::Flat => {
        let w = self.tile_width * 3 / 4;
        let h = self.tile_height / 2;
        ivec2(
          w * hex.q,
          h * (hex.r + hex.r + hex.q),
        )
      },
      HexOrientation::Pointy => {
        let w = self.tile_width / 2;
        let h = self.tile_height * 3 / 4;
        ivec2(
          w * (hex.q + hex.q + hex.r),
          h * hex.r,
        )
      },
    }
  }

  pub fn from_pixel(&self, p: IVec2) -> Hex {
    let sqrt_three = f32::sqrt(3.);
    match self.layout {
      HexOrientation::Flat => {
        let w = self.tile_width as f32 / 2.;
        let h = self.tile_height as f32 / sqrt_three;
        let x = p.x as f32 / w;
        let y = p.y as f32 / h;
        let q = x * ( 2. / 3.);
        let r = x * (-1. / 3.) + y * (sqrt_three / 3.);
        round(q, r)
      },
      HexOrientation::Pointy => {
        let w = self.tile_width as f32 / sqrt_three;
        let h = self.tile_height as f32 / 2.;
        let x = p.x as f32 / w;
        let y = p.y as f32 / h;
        let q = x * (sqrt_three / 3.) + y * (-1. / 3.);
        let r =                         y * ( 2. / 3.);
        round(q, r)
      },
    }
  }
}

fn round(q: f32, r: f32) -> Hex {
  let s = -q - r;
  let mut qi = q.round();
  let mut ri = r.round();
  let mut si = s.round();
  let qd = (qi - q).abs();
  let rd = (ri - r).abs();
  let sd = (si - s).abs();
  if qd > rd && qd > sd {
    qi = -ri - si;
  } else if rd > sd {
    ri = -qi - si;
  } else {
    si = -qi - ri;
  }
  Hex::new(qi as i32, ri as i32, si as i32)
}
