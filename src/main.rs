mod entity;
mod sprite;
mod hex;
mod resources;

use macroquad::prelude::*;
use entity::*;
use sprite::*;
use hex::*;
use resources::*;

use std::collections::HashMap;
use std::collections::HashSet;

#[derive(Copy, Clone, Debug)]
enum ObjectType {
  Ship,
  Torpedo,
  Asteroid,
  //Planet,
  //Gravity0,
  //Gravity1,
  //Gravity2,
  //Gravity3,
  //Gravity4,
  //Gravity5,
}

#[derive(Copy, Clone, Debug)]
struct VisibleObject(ObjectType, Color);

struct Components<T> {
  by_id: [Option<T>; ENTITY_MAX],
}

impl<T> Components<T>{
  const NONE: Option<T> = None;

  pub fn new() -> Self {
    Components {
      by_id: [Self::NONE; ENTITY_MAX],
    }
  }

  pub fn has(&self, entity_id: EntityId) -> bool {
    match self.by_id[entity_id] {
      Some(_) => true,
      None => false,
    }
  }

  pub fn get(&self, entity_id: EntityId) -> &Option<T> {
    &self.by_id[entity_id]
  }

  pub fn get_mut(&mut self, entity_id: EntityId) -> &mut Option<T> {
    &mut self.by_id[entity_id]
  }

  pub fn set(&mut self, entity_id: EntityId, component: T) {
    self.by_id[entity_id] = Some(component);
  }
}

struct Positions {
  components: Components<Hex>,
  by_hex: HashMap<Hex, HashSet<EntityId>>,
}

impl Positions {
  pub fn new() -> Self {
    Positions {
      components: Components::new(),
      by_hex: HashMap::new(),
    }
  }

  pub fn at(&self, hex: Hex) -> Option<&HashSet<EntityId>> {
    self.by_hex.get(&hex)
  }

  pub fn has(&self, entity_id: EntityId) -> bool {
    self.components.has(entity_id)
  }

  pub fn get(&self, entity_id: EntityId) -> &Option<Hex> {
    self.components.get(entity_id)
  }

  pub fn set(&mut self, entity_id: EntityId, hex: Hex) {
    if let Some(previous) = *self.components.get(entity_id) {
      self.by_hex.entry(previous).and_modify(|v| { v.remove(&entity_id); });
    }
    self.components.set(entity_id, hex);
    self.by_hex.entry(hex)
      .and_modify(|v| { v.insert(entity_id); })
      .or_insert_with(|| {
        let mut hs = HashSet::with_capacity(1);
        hs.insert(entity_id);
        hs
      });
  }
}

struct World {
  seed: u32,
  entities: EntityTracker,
  player: Option<EntityId>,
  position: Positions,
  velocity: Components<Hex>,
  history: Components<Vec<Hex>>,
  engine_power: Components<i32>,
  visible_object: Components<VisibleObject>,
  center_hex: Hex,
  drag: Option<Hex>,
  simulate: bool,
  need_simulate: bool,
  turn: i32,
}

impl World {
  fn new() -> Box<Self> {
    Box::new(Self {
      seed: 0,
      entities: EntityTracker::new(),
      player: None,
      position: Positions::new(),
      velocity: Components::new(),
      history: Components::new(),
      engine_power: Components::new(),
      visible_object: Components::new(),
      center_hex: hex(0, 0),
      drag: None,
      need_simulate: false,
      simulate: false,
      turn: 1,
    })
  }
}

fn hash_u32(x: u32) -> u32 {
  let mut v = x;
  v ^= v.wrapping_shr(16);
  v = v.wrapping_mul(0x85ebca6b);
  v ^= v.wrapping_shr(13);
  v = v.wrapping_mul(0xc2b2ae35);
  v ^= v.wrapping_shr(16);
  v
}

fn hash2_u32(x: u32, y: u32) -> u32 {
  let mut v = hash_u32(x);
  v = v.wrapping_add(0x9e3779b9);
  v = v.wrapping_add(y.wrapping_shl(6));
  v = v.wrapping_add(y.wrapping_shr(2));
  y ^ v
}

fn hash_f32(seed: u32, x: f32) -> f32 {
  let x = hash2_u32(seed, x.to_bits());
  let v = hash_u32(x) as f32;
  v / (u32::MAX as f32)
}

fn hash2_f32(seed: u32, x: f32, y: f32) -> f32 {
  let x = hash2_u32(seed, x.to_bits());
  let y = hash2_u32(seed, y.to_bits());
  let v = hash2_u32(x, y) as f32;
  v / (u32::MAX as f32)
}

pub fn hash_vec2(seed: u32, p: Vec2) -> f32 {
  hash2_f32(seed, p.x, p.y)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub fn value_noise(seed: u32, p: Vec2) -> f32 {
  let i = p.floor();
  let f = vec2(p.x - i.x, p.y - i.y);
  let u = f * f * (vec2(3.0, 3.0) - 2.0 * f);
  let a = hash_vec2(seed, i + vec2(0.0, 0.0));
  let b = hash_vec2(seed, i + vec2(1.0, 0.0));
  let c = hash_vec2(seed, i + vec2(0.0, 1.0));
  let d = hash_vec2(seed, i + vec2(1.0, 1.0));
  let ab = lerp(a, b, u.x);
  let cd = lerp(c, d, u.x);
  lerp(ab, cd, u.y)
}

pub fn fbm(seed: u32, p: Vec2) -> f32 {
  let octaves = 6;
  let lacunarity = 2.0;
  let gain = 0.5;
  let mut value = 0.0;
  let mut frequency = 1.0;
  let mut amplitude = 0.5;
  let mut max = 0.0;
  for i in 0..octaves {
    max += amplitude;
    value += amplitude * value_noise(seed, frequency * p);
    frequency *= lacunarity;
    amplitude *= gain;
  }
  value / max
}

fn generate_asteroid(seed: u32, position: Hex) -> f32 {
  let seed = hash2_u32(seed, 4332);
  if (position - hex(0, 0)).len() < 4 {
    return 1.;
  }
  let x = position.q;
  let y = position.r + (x + (x & 1)) / 2;
  let scale = 5.0;
  let p = vec2(x as f32, y as f32) / scale;
  value_noise(seed, p)
}

fn remap(x: f32, xa: f32, xb: f32, ya: f32, yb: f32) -> f32 {
  let t = x / (xb - xa);
  ya + (yb - ya) * t
}

fn create_world() -> Box<World> {
  let mut world = World::new();
  //world.seed = macroquad::rand::rand();
  world.seed = 12;
  {
    let entity_id = world.entities.create();
    let position = hex(0, 0);
    world.position.set(entity_id, position);
    world.velocity.set(entity_id, hex(0, 0));
    world.history.set(entity_id, vec![position]);
    world.visible_object.set(entity_id, VisibleObject(ObjectType::Ship, DARKGREEN));
    world.engine_power.set(entity_id, 1);
    world.player = Some(entity_id);
  }
  {
    let center = world.center_hex;
    for position in center.spiral(25) {
      let p = generate_asteroid(world.seed, position);
      let t = 0.2;
      if p < t {
        let c = 1. - remap(p, 0., t, 0.3, 0.7);
        let color = Color::new(c, c, c, 1.);
        let entity_id = world.entities.create();
        world.position.set(entity_id, position);
        world.visible_object.set(entity_id, VisibleObject(ObjectType::Asteroid, color));
      }
    }
  }
  world
}

fn drag_scroll_system(mut ctx: &mut Context) {
  if ctx.world.simulate {
    return;
  }
  if let Some(start) = ctx.world.drag {
    let current = ctx.resources.hex_view.from_pixel(ctx.cursor_screen);
    ctx.world.center_hex = current - start;
    if is_mouse_button_released(MouseButton::Right) {
      ctx.world.drag = None;
    }
  } else {
    if is_mouse_button_pressed(MouseButton::Right) {
      let current = ctx.resources.hex_view.from_pixel(ctx.cursor_screen);
      ctx.world.drag = Some(current - ctx.world.center_hex);
    }
  }
}

fn player_thrust_system(mut ctx: &mut Context) {
  if ctx.world.simulate {
    return;
  }
  if !is_mouse_button_released(MouseButton::Left) {
    return;
  }
  let player_entity_id = match ctx.world.player {
    Some(x) => x,
    None => return,
  };
  let position = match ctx.world.position.get(player_entity_id) {
    &Some(x) => x,
    &None => return,
  };
  let velocity = match ctx.world.velocity.get(player_entity_id) {
    &Some(x) => x,
    &None => return,
  };
  let engine_power = match ctx.world.engine_power.get(player_entity_id) {
    &Some(x) => x,
    &None => 0,
  };
  let next_position = position + velocity;
  let desired_thrust = (ctx.cursor_world - next_position).len();
  if desired_thrust > engine_power {
    return;
  }
  ctx.world.velocity.set(player_entity_id, ctx.cursor_world - position);
  ctx.world.need_simulate = true;
}

fn velocity_position_system(mut ctx: &mut Context) {
  if !ctx.world.simulate {
    return;
  }
  for entity_id in ctx.world.entities.all() {
    let start_position = match ctx.world.position.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let velocity = match ctx.world.velocity.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let end_position = start_position + velocity;
    for step in start_position.line(end_position) {
      match step {
        Path::One(a) => {
          if a == start_position {
            continue;
          }
          let entities_at = match ctx.world.position.at(a) {
            Some(x) => x,
            None => continue,
          };
          for other in entities_at.to_owned().iter() {
            collision_event(&mut ctx, &entity_id, &other);
          }
        },
        Path::Alt(a, b) => {
          let mut entities_at = match ctx.world.position.at(a) {
            Some(x) => x,
            None => continue,
          };
          if entities_at.len() > 0 {
            entities_at = match ctx.world.position.at(b) {
              Some(x) => x,
              None => continue,
            }
          }
          for other in entities_at.to_owned().iter() {
            collision_event(&mut ctx, &entity_id, &other);
          }
        },
      }
    }
    ctx.world.position.set(entity_id, end_position);
    if let &mut Some(ref mut history) = ctx.world.history.get_mut(entity_id) {
      history.push(end_position);
    }
  }
}

fn simulation_system(mut ctx: &mut Context) {
  ctx.world.simulate = false;
  if ctx.world.need_simulate {
    ctx.world.need_simulate = false;
    ctx.world.simulate = true;
    ctx.world.turn += 1;
  }
}

fn background_hex_grid_system(ctx: &Context) {
  let tw = ctx.resources.hex_view.tile_width as f32;
  let th = ctx.resources.hex_view.tile_height as f32;
  let xn = (screen_width() / tw * 3. / 4.) as i32;
  let yn = f32::ceil(screen_height() / th / 2.) as i32;
  for y in -yn..=yn {
    for x in -xn..=xn {
      let p = x;
      let q = y - (x - (x&1)) / 2;
      let hex = hex(p, q);
      let position = ctx.resources.hex_view.to_pixel(hex);
      ctx.resources.hex_empty.draw(DARKGRAY, MIDDLE_CENTER, position);
    }
  }
}

fn player_cursor_highlight_system(ctx: &Context) {
  let color = Color::new(0., 1., 0., 1.);
  if let Some(player_entity_id) = ctx.world.player {
    let player_hex = match ctx.world.position.get(player_entity_id) {
      Some(x) => x,
      None => return,
    };
    for step in player_hex.line(ctx.cursor_world) {
      match step {
        Path::One(a) => {
          let a = ctx.resources.hex_view.to_pixel(a + ctx.world.center_hex);
          ctx.resources.hex_empty.draw(color, MIDDLE_CENTER, a);
        },
        Path::Alt(a, b) => {
          let a = ctx.resources.hex_view.to_pixel(a + ctx.world.center_hex);
          ctx.resources.hex_empty.draw(color, MIDDLE_CENTER, a);
          let b = ctx.resources.hex_view.to_pixel(b + ctx.world.center_hex);
          ctx.resources.hex_empty.draw(color, MIDDLE_CENTER, b);
        },
      }
    }
  } else {
    let cursor_pix = ctx.resources.hex_view.to_pixel(ctx.cursor_world + ctx.world.center_hex);
    ctx.resources.hex_empty.draw(color, MIDDLE_CENTER, cursor_pix);
  }
}

fn player_velocity_highlight_system(ctx: &Context) {
  let player_entity_id = match ctx.world.player {
    Some(x) => x,
    None => return,
  };
  let position = match ctx.world.position.get(player_entity_id) {
    &Some(x) => x,
    &None => return,
  };
  let velocity = match ctx.world.velocity.get(player_entity_id) {
    &Some(x) => x,
    &None => return,
  };
  let engine_power = match ctx.world.engine_power.get(player_entity_id) {
    &Some(x) => x,
    &None => 0,
  };
  let next_position = position + velocity;
  for neighbor in next_position.spiral(engine_power) {
    let screen_position = ctx.resources.hex_view.to_pixel(neighbor + ctx.world.center_hex);
    ctx.resources.hex_empty.draw(YELLOW, MIDDLE_CENTER, screen_position);
  }
}

fn visible_object_system(ctx: &Context) {
  for entity_id in ctx.world.entities.all() {
    let position = match ctx.world.position.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let visible_object = match ctx.world.visible_object.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let VisibleObject(object_type, color) = visible_object;
    let position = ctx.resources.hex_view.to_pixel(position + ctx.world.center_hex);
    let sprite = match object_type {
      ObjectType::Ship => &ctx.resources.ship,
      ObjectType::Torpedo => &ctx.resources.torpedo,
      ObjectType::Asteroid => &ctx.resources.asteroid,
    };
    sprite.draw(color, MIDDLE_CENTER, position);
  }
}

fn history_trail_system(ctx: &Context) {
  for entity_id in ctx.world.entities.all() {
    let history = match &ctx.world.history.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let visible_object = match ctx.world.visible_object.get(entity_id) {
      &Some(x) => x,
      &None => continue,
    };
    let VisibleObject(_, color) = visible_object;
    for j in 1..history.len() {
      let a = history[j - 1];
      let b = history[j];
      let a = a + ctx.world.center_hex;
      let b = b + ctx.world.center_hex;
      let a = ctx.resources.hex_view.to_pixel(a);
      let b = ctx.resources.hex_view.to_pixel(b);
      let offset = MIDDLE_CENTER.window_offset();
      let a = a + offset;
      let b = b + offset;
      draw_line(a.x as f32, a.y as f32, b.x as f32, b.y as f32, 2., color);
    }
  }
}

struct Context<'a> {
  resources: &'a Resources,
  world: &'a mut World,
  cursor_screen: IVec2,
  cursor_world: Hex,
}

fn tick_event(mut ctx: &mut Context) {
  // control systems
  drag_scroll_system(&mut ctx);
  player_thrust_system(&mut ctx);
  // simulation systems
  velocity_position_system(&mut ctx);
  simulation_system(&mut ctx);
  // Drawing systems
  background_hex_grid_system(&mut ctx);
  player_velocity_highlight_system(&mut ctx);
  player_cursor_highlight_system(&mut ctx);
  visible_object_system(&mut ctx);
  history_trail_system(&mut ctx);
}

fn collision_event(ctx: &mut Context, a: &EntityId, b: &EntityId) {
  println!("collision {:?} <==> {:?}", a, b);
}

fn window_conf() -> Conf {
  Conf {
    window_title: "Rust test".to_owned(),
    high_dpi: true,
    ..Default::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  //let mut min = 0.0;
  //let mut max = 0.0;
  //macroquad::rand::srand(miniquad::date::now().floor() as u64);
  //let o = rand::gen_range(0, 1000);
  //let p = vec2(
    //rand::gen_range(-1000.0, 1000.0),
    //rand::gen_range(-1000.0, 1000.0)
  //);
  //println!("noise {:?}, {:?}", o, p);
  //for i in 0..10000 {
    ////let n = hash_f32((o + i) as f32);
    ////let n = hash_vec2(o + i, p);
    ////let n = value_noise(o + i, p);
    //let n = fbm(o + i, p);
    //if i < 100 {
      //println!("{:?}", n);
    //}
    //if n < min {
      //min = n;
    //}
    //if n > max {
      //max = n;
    //}
  //}
  //println!("noise {:?}, {:?}", min, max);
  let mut ctx = Context {
    resources: &Resources::load(1).await,
    world: &mut create_world(),
    cursor_screen: ivec2(0, 0),
    cursor_world: hex(0, 0),
  };
  loop {
    ctx.cursor_screen = {
      let (x, y) = mouse_position();
      ivec2(x as i32, y as i32) - MIDDLE_CENTER.window_offset()
    };
    ctx.cursor_world = {
      ctx.resources.hex_view.from_pixel(ctx.cursor_screen) - ctx.world.center_hex
    };
    clear_background(BLACK);
    tick_event(&mut ctx);
    next_frame().await
  }
}
