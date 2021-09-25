use macroquad::prelude::{
  FilterMode,
  load_texture,
};
use crate::anchor::MIDDLE_BOTTOM;
use crate::sprite::{
  Sprite,
};
use crate::hex::{
  HexView,
  HexOrientation,
};

pub const HEX_WIDTH: i32 = 32;
pub const HEX_HEIGHT: i32 = 28;
pub const HEX_VIEW: HexView = HexView::new(HexOrientation::Flat, HEX_WIDTH, HEX_HEIGHT);

pub struct Resources {
  pub hex_empty: Sprite,
  pub ship: Sprite,
  pub asteroid: Sprite,
}

impl Resources {
  pub async fn load() -> Resources {
    Resources {
      hex_empty: create_hex_sprite("data/hex-empty.png", 0, 0).await,
      ship: create_hex_sprite("data/ship.png", 0, 0).await,
      asteroid: create_hex_sprite("data/asteroid.png", 0, 0).await,
    }
  }
}

async fn create_hex_sprite(filename: &str, x: i32, y: i32) -> Sprite {
  let tex = load_texture(filename).await.unwrap();
  tex.set_filter(FilterMode::Nearest);
  let mut sprite = Sprite::new(tex, MIDDLE_BOTTOM, HEX_WIDTH, HEX_HEIGHT, x, y);
  // some tiles are taller however all tiles should be centered from their
  // middle bottom edge based on the "nominal" tile size.
  sprite.pivot_y += HEX_HEIGHT / 2;
  sprite
}
