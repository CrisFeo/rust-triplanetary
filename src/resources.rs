use macroquad::prelude::{
  FilterMode,
  load_texture,
  build_textures_atlas,
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
  pub hex_filled: Sprite,
  pub ship: Sprite,
  pub asteroid: Sprite,
  pub pip_open: Sprite,
  pub pip_closed: Sprite,
  pub gravity_arrow_0: Sprite,
  pub gravity_arrow_1: Sprite,
  pub gravity_arrow_2: Sprite,
  pub gravity_arrow_3: Sprite,
  pub gravity_arrow_4: Sprite,
  pub gravity_arrow_5: Sprite,
}

impl Resources {
  pub async fn load() -> Resources {
    let r = Resources {
      hex_empty: create_hex_sprite("hex_empty.png", 0, 0).await,
      hex_filled: create_hex_sprite("hex_filled.png", 0, 0).await,
      ship: create_hex_sprite("ship.png", 0, 0).await,
      asteroid: create_hex_sprite("asteroid.png", 0, 0).await,
      pip_open: create_hex_sprite("pip_open.png", 0, 0).await,
      pip_closed: create_hex_sprite("pip_closed.png", 0, 0).await,
      gravity_arrow_0: create_hex_sprite("gravity_arrow_0.png", 0, 0).await,
      gravity_arrow_1: create_hex_sprite("gravity_arrow_1.png", 0, 0).await,
      gravity_arrow_2: create_hex_sprite("gravity_arrow_2.png", 0, 0).await,
      gravity_arrow_3: create_hex_sprite("gravity_arrow_3.png", 0, 0).await,
      gravity_arrow_4: create_hex_sprite("gravity_arrow_4.png", 0, 0).await,
      gravity_arrow_5: create_hex_sprite("gravity_arrow_5.png", 0, 0).await,
    };
    build_textures_atlas();
    r
  }
}

async fn create_hex_sprite(filename: &str, x: i32, y: i32) -> Sprite {
  let tex = match load_texture(filename).await {
    Ok(x) => x,
    Err(e) => panic!("error loading sprite {}\n{}", filename, e),
  };
  tex.set_filter(FilterMode::Nearest);
  let mut sprite = Sprite::new(tex, MIDDLE_BOTTOM, HEX_WIDTH, HEX_HEIGHT, x, y);
  // some tiles are taller however all tiles should be centered from their
  // middle bottom edge based on the "nominal" tile size.
  sprite.pivot_y += HEX_HEIGHT / 2;
  sprite
}
