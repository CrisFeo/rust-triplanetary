use std::collections::{
  VecDeque,
  HashMap,
};
use triplanetary::*;

macro_rules! must_return {
  ($x:expr) => {
    match $x {
      Some(y) => y,
      None => return,
    }
  };
}

macro_rules! must_continue {
  ($x:expr) => {
    match $x {
      Some(y) => y,
      None => continue,
    }
  };
}

#[derive(Copy, Clone, Debug)]
enum ObjectType {
  Ship,
  Asteroid,
  //Planet,
  Gravity0,
  Gravity1,
  Gravity2,
  Gravity3,
  Gravity4,
  Gravity5,
}

impl std::fmt::Display for ObjectType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Copy, Clone, Debug)]
struct VisibleObject(ObjectType, Color);

#[derive(Copy, Clone, Debug)]
enum Nav {
  Idle,
  GoTo(Hex),
  Seek(EntityId),
}

#[derive(Copy, Clone, Debug)]
struct HistoryEvent {
  position: Hex,
  thrust_applied: i32,
}

#[derive(Copy, Clone, Debug)]
struct Engine {
  power: i32,
  thrust_applied: i32,
}

#[derive(Copy, Clone, Debug)]
struct Camera {
  center_hex: Hex,
  center_pix: IVec2,
  drag: Option<IVec2>,
}

impl Camera {
  pub fn to_screen(&self, hex: Hex) -> IVec2 {
    HEX_VIEW.to_pixel(hex - self.center_hex) - self.center_pix
  }

  pub fn to_world(&self, pix: IVec2) -> Hex {
    HEX_VIEW.from_pixel(pix + self.center_pix) + self.center_hex
  }
}


struct World {
  seed: u32,
  camera: Camera,
  player: Option<EntityId>,
  vision_radius: i32,
  visibility: HashMap<Hex, (bool, bool)>,
  need_simulate: bool,
  turn: i32,
  entities: EntityTracker,
  name: Components<String>,
  position: Positions,
  velocity: Components<Hex>,
  history: Components<VecDeque<HistoryEvent>>,
  engine: Components<Engine>,
  visible_object: Components<VisibleObject>,
  nav: Components<Nav>,
}

impl World {
  fn new() -> Box<Self> {
    Box::new(Self {
      seed: 0,
      camera: Camera {
        center_hex: hex(0, 0),
        center_pix: ivec2(0, 0),
        drag: None,
      },
      player: None,
      vision_radius: 50,
      visibility: HashMap::new(),
      need_simulate: false,
      turn: 1,
      entities: EntityTracker::new(),
      name: Components::new(),
      position: Positions::new(),
      velocity: Components::new(),
      history: Components::new(),
      engine: Components::new(),
      visible_object: Components::new(),
      nav: Components::new(),
    })
  }

  pub fn remove(&mut self, entity_id: EntityId) {
    self.name.del(entity_id);
    self.position.del(entity_id);
    self.velocity.del(entity_id);
    self.history.del(entity_id);
    self.engine.del(entity_id);
    self.visible_object.del(entity_id);
    self.nav.del(entity_id);
    self.entities.remove(entity_id);
  }
}

fn generate_object(seed: u32, position: Hex) -> Option<VisibleObject> {
  let seed = hash2_u32(seed, 4331);
  if (position - hex(0, 0)).len() < 4 {
    return None;
  }
  let scale = 100.;
  let p = HEX_VIEW.to_pixel(position).as_f32() / scale;
  let v = fbm(2, 5., 0.3, seed, p.x, p.y);
  let low = 0.55;
  let high = 1.;
  if v < low || v > high {
    return None;
  }
  let c = remap(v, low, high, 0.3, 0.8);
  let c = Color{ r: c, g: c, b: c, a: 1. };
  Some(VisibleObject(ObjectType::Asteroid, c))
}

fn input_mouse_pan_system(ctx: &mut Context) {
  let camera = &mut ctx.world.camera;
  if let Some(start) = camera.drag {
    camera.center_pix = start - ctx.cursor_screen;
    if is_mouse_button_released(MouseButton::Right) {
      let offset_hex = HEX_VIEW.from_pixel(camera.center_pix);
      let offset_pix = HEX_VIEW.to_pixel(offset_hex);
      camera.center_hex = camera.center_hex + offset_hex;
      camera.center_pix = camera.center_pix - offset_pix;
      camera.drag = None;
    }
  } else {
    if is_mouse_button_pressed(MouseButton::Right) {
      camera.drag = Some(ctx.cursor_screen + camera.center_pix);
    }
  }
}

fn input_player_thrust_system(mut ctx: &mut Context) {
  if is_mouse_button_released(MouseButton::Left) {
    let player_entity_id = must_return!(ctx.world.player);
    let position = *must_return!(ctx.world.position.get(player_entity_id));
    let velocity = *must_return!(ctx.world.velocity.get(player_entity_id));
    let mut engine = *must_return!(ctx.world.engine.get(player_entity_id));
    let next_position = position + velocity;
    let desired_thrust = (ctx.cursor_world - next_position).len();
    if desired_thrust > engine.power {
      return;
    }
    ctx.world.engine.set(player_entity_id, {
      engine.thrust_applied = desired_thrust;
      engine
    });
    ctx.world.velocity.set(player_entity_id, ctx.cursor_world - position);
    ctx.world.need_simulate = true;
  }
  if is_key_released(KeyCode::Space) {
    ctx.world.need_simulate = true;
  }
}

fn simulate_nav_system(ctx: &mut Context) {
  for entity_id in ctx.world.entities.all() {
    let nav = *must_continue!(ctx.world.nav.get(entity_id));
    let position = *must_continue!(ctx.world.position.get(entity_id));
    let velocity = *must_continue!(ctx.world.velocity.get(entity_id));
    let mut engine = *must_continue!(ctx.world.engine.get(entity_id));
    let desired_velocity = match nav {
      Nav::Idle => {
        hex(0, 0)
      },
      Nav::GoTo(target_position) => {
        let target_vector = target_position - position;
        let target_length = target_vector.len() as f32;
        let desired_speed = target_length.sqrt().floor() as usize;
        match hex(0, 0).line(target_vector).get(desired_speed) {
          Some(&Path::One(x)) => x,
          Some(&Path::Alt(x, _)) => x,
          None => velocity,
        }
      },
      Nav::Seek(target_entity_id) => {
        let target_position = *must_continue!(ctx.world.position.get(target_entity_id));
        let target_velocity = match ctx.world.velocity.get(target_entity_id) {
          Some(x) => *x,
          None => hex(0, 0),
        };
        let mut a_position = position;
        let mut b_position = target_position + target_velocity;
        let mut a_velocity = velocity;
        while a_velocity != target_velocity {
          a_velocity = a_velocity.move_to(target_velocity, engine.power);
          a_position = a_position + a_velocity;
          b_position = b_position + target_velocity;
        }
        b_position - a_position
      },
    };
    let new_velocity = velocity.move_to(desired_velocity, engine.power);
    ctx.world.engine.set(entity_id, {
      engine.thrust_applied = (new_velocity - velocity).len();
      engine
    });
    ctx.world.velocity.set(entity_id, new_velocity);
  }
}

fn simulate_movement_system(mut ctx: &mut Context) {
  for entity_id in ctx.world.entities.all() {
    let start_position = *must_continue!(ctx.world.position.get(entity_id));
    let velocity = *must_continue!(ctx.world.velocity.get(entity_id));
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
            simulate_collision_event(&mut ctx, entity_id, *other);
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
            simulate_collision_event(&mut ctx, entity_id, *other);
          }
        },
      }
    }
    ctx.world.position.set(entity_id, end_position);
    if let Some(ref mut history) = ctx.world.history.get_mut(entity_id) {
      if let Some(engine) = ctx.world.engine.get(entity_id) {
        if let Some(entry) = history.back_mut() {
          entry.thrust_applied = engine.thrust_applied;
        }
      }
      history.push_back(HistoryEvent {
        position: end_position,
        thrust_applied: 0,
      });
      if history.len() > 6 {
        history.pop_front();
      }
    }
    if let Some(engine) = ctx.world.engine.get(entity_id) {
      let mut engine = *engine;
      ctx.world.engine.set(entity_id, {
        engine.thrust_applied = 0;
        engine
      });
    }
  }
}

fn simulate_generate_around_player_system(ctx: &mut Context) {
  let player_entity_id = must_return!(ctx.world.player);
  let current = match ctx.world.history.get(player_entity_id) {
    Some(h) => match h.get(h.len() - 1) {
      Some(x) => x.position,
      None => return,
    },
    None => return,
  };
  let previous = match ctx.world.history.get(player_entity_id) {
    Some(h) if h.len() > 1 => h.get(h.len() - 2).map(|x| x.position),
    _ => None,
  };
  if let Some(previous) = previous {
    if current == previous {
      return;
    }
    for position in previous.spiral(ctx.world.vision_radius) {
      if (position - current).len() <= ctx.world.vision_radius {
        continue;
      }
      let entities_at = must_continue!(ctx.world.position.at(position));
      for entity_id in entities_at.to_owned().iter() {
        ctx.world.remove(*entity_id);
      }
    }
  }
  for position in current.spiral(ctx.world.vision_radius) {
    if let Some(previous) = previous {
      if (position - previous).len() <= ctx.world.vision_radius {
        continue;
      }
    }
    let result = generate_object(ctx.world.seed, position);
    if let Some(visible_object) = result {
      let entity_id = ctx.world.entities.create();
      ctx.world.position.set(entity_id, position);
      ctx.world.visible_object.set(entity_id, visible_object);
    }
  }
}

fn simulate_vision_system(ctx: &mut Context) {
  let player_entity_id = must_return!(ctx.world.player);
  let origin = *must_return!(ctx.world.position.get(player_entity_id));
  let visibility = &mut ctx.world.visibility;
  visibility.clear();
  visibility.insert(origin, (true, false));
  for position in origin.spiral(ctx.world.vision_radius) {
    let is_visible_through = |position| {
      let (visible, obstructed) = *visibility.get(&position).unwrap();
      visible && !obstructed
    };
    let visible = match position.line(origin).get(1) {
      Some(&Path::One(a)) => is_visible_through(a),
      Some(&Path::Alt(a, b)) => is_visible_through(a) && is_visible_through(b),
      None => true,
    };
    let obstructed = match ctx.world.position.at(position) {
      Some(x) => x.len() > 0,
      None => false,
    };
    visibility.insert(position, (visible, obstructed));
  }
}

fn simulate_step_system(mut ctx: &mut Context) {
  if !ctx.world.need_simulate {
    return;
  }
  simulate_event(&mut ctx);
  ctx.world.need_simulate = false;
  ctx.world.turn += 1;
}

fn draw_background_hex_grid_system(ctx: &Context) {
  let tw = HEX_WIDTH as f32;
  let th = HEX_HEIGHT as f32;
  let xn = (screen_width() / tw * 3. / 4.) as i32;
  let yn = f32::ceil(screen_height() / th / 2.) as i32;
  let player_hex = match ctx.world.player {
    Some(player_entity_id) => ctx.world.position.get(player_entity_id),
    None => None,
  };
  let center_hex = ctx.world.camera.to_world(ivec2(0, 0));
  for y in -yn..=yn {
    for x in -xn..=xn {
      let p = x;
      let q = y - (x - (x&1)) / 2;
      let world_hex = center_hex + hex(p, q);
      if let Some(player_hex) = player_hex {
        if (world_hex - *player_hex).len() > ctx.world.vision_radius {
          continue;
        }
      }
      if let Some((visible, _)) = ctx.world.visibility.get(&world_hex) {
        if !*visible {
          continue;
        }
      }
      let color = Color::new(0.1, 0.1, 0.2, 1.);
      let position = ctx.world.camera.to_screen(world_hex);
      ctx.resources.hex_filled.draw(color, position);
    }
  }
}

fn draw_visible_objects_system(ctx: &Context) {
  for entity_id in ctx.world.entities.all().rev() {
    let position = *must_continue!(ctx.world.position.get(entity_id));
    let visible_object = *must_continue!(ctx.world.visible_object.get(entity_id));
    let VisibleObject(object_type, color) = visible_object;
    let position = ctx.world.camera.to_screen(position);
    let sprite = match object_type {
      ObjectType::Ship => &ctx.resources.ship,
      ObjectType::Asteroid => &ctx.resources.asteroid,
      ObjectType::Gravity0 => &ctx.resources.gravity_arrow_0,
      ObjectType::Gravity1 => &ctx.resources.gravity_arrow_1,
      ObjectType::Gravity2 => &ctx.resources.gravity_arrow_2,
      ObjectType::Gravity3 => &ctx.resources.gravity_arrow_3,
      ObjectType::Gravity4 => &ctx.resources.gravity_arrow_4,
      ObjectType::Gravity5 => &ctx.resources.gravity_arrow_5,
    };
    sprite.draw(color, position);
  }
}

fn draw_history_trail_system(ctx: &Context) {
  for entity_id in ctx.world.entities.all() {
    let history = must_continue!(ctx.world.history.get(entity_id));
    let visible_object = *must_continue!(ctx.world.visible_object.get(entity_id));
    let VisibleObject(_, color) = visible_object;
    for j in 1..history.len() {
      let event_a = history[j - 1];
      let event_b = history[j];
      let a = ctx.world.camera.to_screen(event_a.position);
      let b = ctx.world.camera.to_screen(event_b.position);
      draw_line(color, 2, a, b);
      let pip = if event_a.thrust_applied > 0 {
        &ctx.resources.pip_closed
      } else {
        &ctx.resources.pip_open
      };
      pip.draw(color, a);
    }
    let position = *must_continue!(ctx.world.position.get(entity_id));
    let velocity = *must_continue!(ctx.world.velocity.get(entity_id));
    let next_position = position + velocity;
    let a = ctx.world.camera.to_screen(position);
    let b = ctx.world.camera.to_screen(next_position);
    draw_line(color, 3, a, b);
  }
}

fn draw_player_thrust_destination_system(ctx: &Context) {
  let player_entity_id = must_return!(ctx.world.player);
  let position = *must_return!(ctx.world.position.get(player_entity_id));
  let velocity = *must_return!(ctx.world.velocity.get(player_entity_id));
  let engine = *must_return!(ctx.world.engine.get(player_entity_id));
  let next_position = position + velocity;
  for neighbor in next_position.spiral(engine.power) {
    let screen_position = ctx.world.camera.to_screen(neighbor);
    ctx.resources.hex_empty.draw(DARK_YELLOW, screen_position);
  }
}

fn draw_player_to_cursor_hexes_system(ctx: &Context) {
  let color = GREEN;
  if let Some(player_entity_id) = ctx.world.player {
    let player_hex = must_return!(ctx.world.position.get(player_entity_id));
    for step in player_hex.line(ctx.cursor_world) {
      match step {
        Path::One(a) => {
          let a = ctx.world.camera.to_screen(a);
          ctx.resources.hex_empty.draw(color, a);
        },
        Path::Alt(a, b) => {
          let a = ctx.world.camera.to_screen(a);
          let b = ctx.world.camera.to_screen(b);
          ctx.resources.hex_empty.draw(color, a);
          ctx.resources.hex_empty.draw(color, b);
        },
      }
    }
  } else {
    let cursor_pix = ctx.world.camera.to_screen(ctx.cursor_world);
    ctx.resources.hex_empty.draw(color, cursor_pix);
  }
}

struct Context {
  resources: Resources,
  world: Box<World>,
  cursor_screen: IVec2,
  cursor_world: Hex,
}

fn initialize(ctx: &mut Context) {
  let mut world = &mut ctx.world;
  //world.seed = macroquad::rand::rand();
  world.seed = 12;
  let player_entity_id = world.entities.create();
  let player_starting_position = hex(0, 0);
  fn new_history(position: Hex) -> VecDeque<HistoryEvent> {
    vec![
      HistoryEvent {
        position: position,
        thrust_applied: 0,
      }
    ].into_iter().collect()
  }
  {
    world.name.set(player_entity_id, "player".to_string());
    world.position.set(player_entity_id, player_starting_position);
    world.velocity.set(player_entity_id, hex(0, 0));
    world.history.set(player_entity_id, new_history(player_starting_position));
    world.visible_object.set(player_entity_id, VisibleObject(ObjectType::Ship, GREEN));
    world.engine.set(player_entity_id, Engine { power: 1, thrust_applied: 0 });
    world.player = Some(player_entity_id);
  }
  {
    let enemy_starting_position = hex(-20, 20);
    let entity_id = world.entities.create();
    world.name.set(entity_id, "enemy".to_string());
    world.position.set(entity_id, enemy_starting_position);
    world.velocity.set(entity_id, hex(0, 0));
    world.history.set(entity_id, new_history(enemy_starting_position));
    world.visible_object.set(entity_id, VisibleObject(ObjectType::Ship, RED));
    world.engine.set(entity_id, Engine { power: 1, thrust_applied: 0 });
    //world.nav.set(entity_id, Nav::Idle);
    //world.nav.set(entity_id, Nav::GoTo(hex(-15, 15)));
    world.nav.set(entity_id, Nav::Seek(player_entity_id));
  }
  {
    let planet_position =hex(4, -1);
    let mut add_gravity = |direction| {
      let entity_id = world.entities.create();
      world.position.set(entity_id, planet_position + Hex::direction(direction));
      let object_type = match direction {
        3 => ObjectType::Gravity0,
        4 => ObjectType::Gravity1,
        5 => ObjectType::Gravity2,
        0 => ObjectType::Gravity3,
        1 => ObjectType::Gravity4,
        2 => ObjectType::Gravity5,
        _ => panic!(),
      };
      world.visible_object.set(entity_id, VisibleObject(object_type, WHITE));
    };
    add_gravity(0);
    add_gravity(1);
    add_gravity(2);
    add_gravity(3);
    add_gravity(4);
    add_gravity(5);
  }
  simulate_generate_around_player_system(ctx);
  simulate_vision_system(ctx);
}

fn tick_event(mut ctx: &mut Context) {
  // input systems
  input_mouse_pan_system(&mut ctx);
  input_player_thrust_system(&mut ctx);
  // simulation systems
  simulate_step_system(&mut ctx);
  // Drawing systems
  draw_background_hex_grid_system(&mut ctx);
  draw_history_trail_system(&mut ctx);
  draw_visible_objects_system(&mut ctx);
  draw_player_thrust_destination_system(&mut ctx);
  draw_player_to_cursor_hexes_system(&mut ctx);
}

fn simulate_event(mut ctx: &mut Context) {
  simulate_nav_system(&mut ctx);
  simulate_movement_system(&mut ctx);
  simulate_generate_around_player_system(&mut ctx);
  simulate_vision_system(&mut ctx);
}

fn simulate_collision_event(ctx: &mut Context, a: EntityId, b: EntityId) {
  let VisibleObject(type_b, _) = match ctx.world.visible_object.get(b) {
    Some(x) => *x,
    None => return,
  };
  match type_b {
    ObjectType::Ship => simulate_collision_ship(ctx, a),
    ObjectType::Asteroid => simulate_collision_asteroid(ctx, a),
    ObjectType::Gravity0 => simulate_collision_gravity(ctx, 0, a),
    ObjectType::Gravity1 => simulate_collision_gravity(ctx, 1, a),
    ObjectType::Gravity2 => simulate_collision_gravity(ctx, 2, a),
    ObjectType::Gravity3 => simulate_collision_gravity(ctx, 3, a),
    ObjectType::Gravity4 => simulate_collision_gravity(ctx, 4, a),
    ObjectType::Gravity5 => simulate_collision_gravity(ctx, 5, a),
  }
}

fn simulate_collision_asteroid(ctx: &mut Context, entity_id: EntityId) {
  match ctx.world.name.get(entity_id) {
    Some(x) => println!("{} collided with an asteroid", x),
    None => println!("{} collided with an asteroid", entity_id.to_string()),
  };
}

fn simulate_collision_ship(ctx: &mut Context, entity_id: EntityId) {
  match ctx.world.name.get(entity_id) {
    Some(x) => println!("{} collided with a ship", x),
    None => println!("{} collided with a ship", entity_id.to_string()),
  };
}

fn simulate_collision_gravity(ctx: &mut Context, direction: i32, entity_id: EntityId) {
  match ctx.world.name.get(entity_id) {
    Some(x) => println!("{} passed through gravity well", x),
    None => println!("{} passed through gravity well", entity_id.to_string()),
  };
  let velocity = match ctx.world.velocity.get(entity_id) {
    Some(x) => *x,
    None => return,
  };
  ctx.world.velocity.set(entity_id, velocity + Hex::direction(direction));
}

fn window_conf() -> Conf {
  Conf {
    window_title: "Triplanetary".to_owned(),
    high_dpi: true,
    ..Default::default()
  }
}

#[macroquad::main(window_conf)]
async fn main() {
  let mut ctx = Context {
    resources: Resources::load().await,
    world: World::new(),
    cursor_screen: ivec2(0, 0),
    cursor_world: hex(0, 0),
  };
  initialize(&mut ctx);
  loop {
    ctx.cursor_screen = {
      let (x, y) = mouse_position();
      ivec2(x as i32, y as i32) - MIDDLE_CENTER.window_offset()
    };
    ctx.cursor_world = ctx.world.camera.to_world(ctx.cursor_screen);
    clear_background(BLACK);
    tick_event(&mut ctx);
    macroquad::text::draw_text(
      &macroquad::time::get_fps().to_string(),
      30.,
      30.,
      30.,
      GREEN
    );
    next_frame().await
  }
}
