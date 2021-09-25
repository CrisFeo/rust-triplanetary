use triplanetary::*;

#[derive(Copy, Clone, Debug)]
enum ObjectType {
  Ship,
  Asteroid,
  //Planet,
  //Gravity0,
  //Gravity1,
  //Gravity2,
  //Gravity3,
  //Gravity4,
  //Gravity5,
}

impl std::fmt::Display for ObjectType {
  fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
    write!(f, "{:?}", self)
  }
}

#[derive(Copy, Clone, Debug)]
struct VisibleObject(ObjectType, Color);

struct World {
  seed: u32,
  entities: EntityTracker,
  name: Components<String>,
  player: Option<EntityId>,
  vision_radius: i32,
  position: Positions,
  velocity: Components<Hex>,
  history: Components<Vec<Hex>>,
  engine_power: Components<i32>,
  visible_object: Components<VisibleObject>,
  center_hex: Hex,
  center_pix: IVec2,
  drag: Option<IVec2>,
  need_simulate: bool,
  turn: i32,
}

impl World {
  fn new() -> Box<Self> {
    Box::new(Self {
      seed: 0,
      entities: EntityTracker::new(),
      name: Components::new(),
      player: None,
      vision_radius: 100,
      position: Positions::new(),
      velocity: Components::new(),
      history: Components::new(),
      engine_power: Components::new(),
      visible_object: Components::new(),
      center_hex: hex(0, 0),
      center_pix: ivec2(0, 0),
      drag: None,
      need_simulate: false,
      turn: 1,
    })
  }
}

fn generate_object(seed: u32, position: Hex) -> Option<VisibleObject> {
  let seed = hash2_u32(seed, 4332);
  if (position - hex(0, 0)).len() < 4 {
    return None;
  }
  let x = position.q;
  let y = position.r + (x + (x & 1)) / 2;
  let scale = 5.0;
  let p = vec2(x as f32, y as f32) / scale;
  let v = value_noise(seed, p);
  let v = remap(v, 0., 1., -1., 1.).abs();
  let low = 0.1;
  let high = 0.25;
  if v < low || v > high {
    return None;
  }
  let c = remap(v, low, high, 0.3, 0.6);
  let color = Color::new(c, c, c, 1.);
  Some(VisibleObject(ObjectType::Asteroid, color))
}

fn input_mouse_pan_system(mut ctx: &mut Context) {
  if let Some(start) = ctx.world.drag {
    ctx.world.center_pix = start - ctx.cursor_screen;
    if is_mouse_button_released(MouseButton::Right) {
      let offset_hex = HEX_VIEW.from_pixel(ctx.world.center_pix);
      let offset_pix = HEX_VIEW.to_pixel(offset_hex);
      ctx.world.center_hex = ctx.world.center_hex + offset_hex;
      ctx.world.center_pix = ctx.world.center_pix - offset_pix;
      ctx.world.drag = None;
    }
  } else {
    if is_mouse_button_pressed(MouseButton::Right) {
      ctx.world.drag = Some(ctx.cursor_screen + ctx.world.center_pix);
    }
  }
}

fn input_player_thrust_system(mut ctx: &mut Context) {
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

fn simulate_movement_system(mut ctx: &mut Context) {
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
    if let &mut Some(ref mut history) = ctx.world.history.get_mut(entity_id) {
      history.push(end_position);
    }
  }
}

fn simulate_generate_around_player_system(ctx: &mut Context) {
  let player_entity_id = match ctx.world.player {
    Some(x) => x,
    None => return,
  };
  let current = match ctx.world.history.get(player_entity_id) {
    Some(h) => match h[..] {
      [.., x] => x,
      _ => return,
    },
    None => return,
  };
  let previous = match ctx.world.history.get(player_entity_id) {
    Some(h) => match h[..] {
      [.., x, _] => Some(x),
      _ => None,
    },
    None => None,
  };
  if let Some(previous) = previous {
    if current == previous {
      return;
    }
    for position in previous.spiral(ctx.world.vision_radius) {
      if (position - current).len() <= ctx.world.vision_radius {
        continue;
      }
      let entities_at = match ctx.world.position.at(position) {
        Some(x) => x,
        None => continue,
      };
      for entity_id in entities_at.to_owned().iter() {
        ctx.world.position.del(*entity_id);
        ctx.world.visible_object.del(*entity_id);
        ctx.world.entities.remove(*entity_id);
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
    Some(player_entity_id) => *ctx.world.position.get(player_entity_id),
    None => None,
  };
  let center_hex = screen_to_world(ctx, ivec2(0, 0));
  for y in -yn..=yn {
    for x in -xn..=xn {
      let p = x;
      let q = y - (x - (x&1)) / 2;
      //let world_hex = hex(p, q) + ctx.world.center_hex;
      let world_hex = center_hex + hex(p, q);
      let color = match player_hex {
        Some(player_hex) => {
          if (world_hex - player_hex).len() > ctx.world.vision_radius {
            DARKGRAY
          } else {
            DARKBLUE
          }
        },
        None => DARKGRAY,
      };
      let position = world_to_screen(ctx, world_hex);
      ctx.resources.hex_empty.draw(color, position);
    }
  }
}

fn draw_player_to_cursor_hexes_system(ctx: &Context) {
  let color = Color::new(0., 1., 0., 1.);
  if let Some(player_entity_id) = ctx.world.player {
    let player_hex = match ctx.world.position.get(player_entity_id) {
      Some(x) => x,
      None => return,
    };
    for step in player_hex.line(ctx.cursor_world) {
      match step {
        Path::One(a) => {
          let a = world_to_screen(ctx, a);
          ctx.resources.hex_empty.draw(color, a);
        },
        Path::Alt(a, b) => {
          let a = world_to_screen(ctx, a);
          let b = world_to_screen(ctx, b);
          ctx.resources.hex_empty.draw(color, a);
          ctx.resources.hex_empty.draw(color, b);
        },
      }
    }
  } else {
    let cursor_pix = world_to_screen(ctx, ctx.cursor_world);
    ctx.resources.hex_empty.draw(color, cursor_pix);
  }
}

fn draw_player_thrust_destination_system(ctx: &Context) {
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
    let screen_position = world_to_screen(ctx, neighbor);
    ctx.resources.hex_empty.draw(YELLOW, screen_position);
  }
}

fn draw_visible_objects_system(ctx: &Context) {
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
    let position = world_to_screen(ctx, position);
    let sprite = match object_type {
      ObjectType::Ship => &ctx.resources.ship,
      ObjectType::Asteroid => &ctx.resources.asteroid,
    };
    sprite.draw(color, position);
  }
}

fn draw_history_trail_system(ctx: &Context) {
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
      let a = world_to_screen(ctx, a);
      let b = world_to_screen(ctx, b);
      draw_line(color, 2, a, b);
    }
    if let Some(velocity) = ctx.world.velocity.get(entity_id) {
      if let Some(last) = history.last() {
        let next = *last + *velocity;
        let a = world_to_screen(ctx, *last);
        let b = world_to_screen(ctx, next);
        draw_line(YELLOW, 2, a, b);
      }
    }
  }
}

fn world_to_screen(ctx: &Context, hex: Hex) -> IVec2 {
  HEX_VIEW.to_pixel(hex - ctx.world.center_hex) - ctx.world.center_pix
}

fn screen_to_world(ctx: &Context, pix: IVec2) -> Hex {
  HEX_VIEW.from_pixel(pix + ctx.world.center_pix) + ctx.world.center_hex
}

struct Context<'a> {
  resources: &'a Resources,
  world: &'a mut World,
  cursor_screen: IVec2,
  cursor_world: Hex,
}

fn initialize<'a>(ctx: &mut Context<'a>) {
  let mut world = &mut ctx.world;
  //world.seed = macroquad::rand::rand();
  world.seed = 12;
  let player_starting_position = hex(0, 0);
  {
    let entity_id = world.entities.create();
    world.name.set(entity_id, "player".to_string());
    world.position.set(entity_id, player_starting_position);
    world.velocity.set(entity_id, hex(0, 0));
    world.history.set(entity_id, vec![player_starting_position]);
    world.visible_object.set(entity_id, VisibleObject(ObjectType::Ship, DARKGREEN));
    world.engine_power.set(entity_id, 1);
    world.player = Some(entity_id);
  }
  {
    for position in player_starting_position.spiral(world.vision_radius) {
      let result = generate_object(world.seed, position);
      if let Some(visible_object) = result {
        let VisibleObject(object_type, _) = visible_object;
        let entity_id = world.entities.create();
        let name = object_type.to_string();
        world.name.set(entity_id, name);
        world.position.set(entity_id, position);
        world.visible_object.set(entity_id, visible_object);
      }
    }
  }
}

fn tick_event(mut ctx: &mut Context) {
  // input systems
  input_mouse_pan_system(&mut ctx);
  input_player_thrust_system(&mut ctx);
  // simulation systems
  simulate_step_system(&mut ctx);
  // Drawing systems
  draw_background_hex_grid_system(&mut ctx);
  draw_player_thrust_destination_system(&mut ctx);
  draw_player_to_cursor_hexes_system(&mut ctx);
  draw_visible_objects_system(&mut ctx);
  draw_history_trail_system(&mut ctx);
}

fn simulate_event(mut ctx: &mut Context) {
  simulate_movement_system(&mut ctx);
  simulate_generate_around_player_system(&mut ctx);
}

fn simulate_collision_event(ctx: &mut Context, a: EntityId, b: EntityId) {
  let VisibleObject(type_b, _) = match ctx.world.visible_object.get(b) {
    &Some(x) => x,
    &None => return,
  };
  match type_b {
    ObjectType::Ship => simulate_collision_ship(ctx, a),
    ObjectType::Asteroid => simulate_collision_asteroid(ctx, a),
  }
}

fn simulate_collision_asteroid(ctx: &mut Context, entity_id: EntityId) {
  match ctx.world.name.get(entity_id) {
    Some(x) => println!("{} collided with an asteroid", x),
    &None => println!("{} collided with an asteroid", entity_id.to_string()),
  };
}

fn simulate_collision_ship(ctx: &mut Context, entity_id: EntityId) {
  match ctx.world.name.get(entity_id) {
    Some(x) => println!("{} collided with a ship", x),
    &None => println!("{} collided with a ship", entity_id.to_string()),
  };
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
    resources: &Resources::load().await,
    world: &mut World::new(),
    cursor_screen: ivec2(0, 0),
    cursor_world: hex(0, 0),
  };
  initialize(&mut ctx);
  loop {
    ctx.cursor_screen = {
      let (x, y) = mouse_position();
      ivec2(x as i32, y as i32) - MIDDLE_CENTER.window_offset()
    };
    ctx.cursor_world = screen_to_world(&ctx, ctx.cursor_screen);
    clear_background(BLACK);
    tick_event(&mut ctx);
    next_frame().await
  }
}
