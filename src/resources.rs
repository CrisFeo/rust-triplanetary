use macroquad::prelude::*;
use crate::sprite::*;
use crate::hex::*;

const HEX_WIDTH: i32 = 32;
const HEX_HEIGHT: i32 = 28;

pub struct Resources {
  pub scale: i32,
  pub hex_view: HexView,
  pub hex_empty: Sprite,
  pub ship: Sprite,
  pub torpedo: Sprite,
  pub asteroid: Sprite,
}

impl Resources {
  pub async fn load(scale: i32) -> Resources {
    let hex_view = HexView::new(HexOrientation::Flat, HEX_WIDTH * scale, HEX_HEIGHT * scale);
    Resources {
      scale,
      hex_view,
      hex_empty: create_hex_sprite("data/hex-empty.png", scale, 0, 0).await,
      ship: create_hex_sprite("data/ship.png", scale, 0, 0).await,
      torpedo: create_hex_sprite("data/torpedo.png", scale, 0, 0).await,
      asteroid: create_hex_sprite("data/asteroid.png", scale, 0, 0).await,
    }
  }
}

async fn create_hex_sprite(filename: &str, scale: i32, x: i32, y: i32) -> Sprite {
  let tex = load_texture(filename).await.unwrap();
  tex.set_filter(FilterMode::Nearest);
  let mut sprite = Sprite::new(tex, MIDDLE_BOTTOM, scale, HEX_WIDTH, HEX_HEIGHT, x, y);
  // some tiles are taller however all tiles should be centered from their
  // middle bottom edge based on the "nominal" tile size.
  sprite.pivot_y += (HEX_HEIGHT * scale) / 2;
  sprite
}
