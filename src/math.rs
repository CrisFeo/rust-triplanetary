use macroquad::prelude::{
  Vec2,
  vec2,
};

pub fn hash_u32(x: u32) -> u32 {
  let mut v = x;
  v ^= v.wrapping_shr(16);
  v = v.wrapping_mul(0x85ebca6b);
  v ^= v.wrapping_shr(13);
  v = v.wrapping_mul(0xc2b2ae35);
  v ^= v.wrapping_shr(16);
  v
}

pub fn hash2_u32(x: u32, y: u32) -> u32 {
  let mut v = hash_u32(x);
  v = v.wrapping_add(0x9e3779b9);
  v = v.wrapping_add(y.wrapping_shl(6));
  v = v.wrapping_add(y.wrapping_shr(2));
  y ^ v
}

pub fn hash_f32(seed: u32, x: f32) -> f32 {
  let x = hash2_u32(seed, x.to_bits());
  let v = hash_u32(x) as f32;
  v / (u32::MAX as f32)
}

pub fn hash2_f32(seed: u32, x: f32, y: f32) -> f32 {
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

pub fn remap(x: f32, xa: f32, xb: f32, ya: f32, yb: f32) -> f32 {
  let t = x / (xb - xa);
  ya + (yb - ya) * t
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
  for _ in 0..octaves {
    max += amplitude;
    value += amplitude * value_noise(seed, frequency * p);
    frequency *= lacunarity;
    amplitude *= gain;
  }
  value / max
}
