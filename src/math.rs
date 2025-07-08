pub const fn hash_u32(x: u32) -> u32 {
  let mut v = x;
  v ^= v.wrapping_shr(16);
  v = v.wrapping_mul(0x85ebca6b);
  v ^= v.wrapping_shr(13);
  v = v.wrapping_mul(0xc2b2ae35);
  v ^= v.wrapping_shr(16);
  v
}

pub const fn hash2_u32(x: u32, y: u32) -> u32 {
  let mut v = hash_u32(x);
  v = v.wrapping_add(0x9e3779b9);
  v = v.wrapping_add(y.wrapping_shl(6));
  v = v.wrapping_add(y.wrapping_shr(2));
  y ^ v
}

pub fn hash_f32(seed: u32, x: f32) -> f32 {
  let v = hash2_u32(seed, x.to_bits()) as f32;
  v / (u32::MAX as f32)
}

pub fn hash2_f32(seed: u32, x: f32, y: f32) -> f32 {
  let v = hash2_u32(y.to_bits(), x.to_bits());
  let v = hash2_u32(seed, v) as f32;
  v / (u32::MAX as f32)
}

pub fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub fn clamp(x: f32, a: f32, b: f32) -> f32 {
  if x < a {
    a
  } else if x > b {
    b
  } else {
    x
  }
}

pub fn remap(x: f32, xa: f32, xb: f32, ya: f32, yb: f32) -> f32 {
  let t = (x - xa) / (xb - xa);
  let t = clamp(t, 0., 1.);
  ya + ((yb - ya) * t)
}

pub fn perlin(seed: u32, x: f32, y: f32) -> f32 {
  fn noise(seed: u32, x: i32, y: i32) -> f32 {
    hash2_f32(seed, x as f32, y as f32)
  }
  let x_int = x.floor() as i32;
  let y_int = y.floor() as i32;
  let x_frac = x - x.floor();
  let y_frac = y - y.floor();
  let n01 = noise(seed, x_int - 1, y_int - 1);
  let n02 = noise(seed, x_int + 1, y_int - 1);
  let n03 = noise(seed, x_int - 1, y_int + 1);
  let n04 = noise(seed, x_int + 1, y_int + 1);
  let n05 = noise(seed, x_int - 1, y_int);
  let n06 = noise(seed, x_int + 1, y_int);
  let n07 = noise(seed, x_int, y_int - 1);
  let n08 = noise(seed, x_int, y_int + 1);
  let n09 = noise(seed, x_int, y_int);
  let n12 = noise(seed, x_int + 2, y_int - 1);
  let n14 = noise(seed, x_int + 2, y_int + 1);
  let n16 = noise(seed, x_int + 2, y_int);
  let n23 = noise(seed, x_int - 1, y_int + 2);
  let n24 = noise(seed, x_int + 1, y_int + 2);
  let n28 = noise(seed, x_int, y_int + 2);
  let n34 = noise(seed, x_int + 2, y_int + 2);
  let x0y0 = 0.0625 * (n01 + n02 + n03 + n04) + 0.125 * (n05 + n06 + n07 + n08) + 0.25 * (n09);
  let x1y0 = 0.0625 * (n07 + n12 + n08 + n14) + 0.125 * (n09 + n16 + n02 + n04) + 0.25 * (n06);
  let x0y1 = 0.0625 * (n05 + n06 + n23 + n24) + 0.125 * (n03 + n04 + n09 + n28) + 0.25 * (n08);
  let x1y1 = 0.0625 * (n09 + n16 + n28 + n34) + 0.125 * (n08 + n14 + n06 + n24) + 0.25 * (n04);
  let v1 = lerp(x0y0, x1y0, x_frac);
  let v2 = lerp(x0y1, x1y1, x_frac);
  lerp(v1, v2, y_frac)
}

pub fn fbm(octaves: i32, lacunarity: f32, gain: f32, seed: u32, x: f32, y: f32) -> f32 {
  let mut value = 0.0;
  let mut frequency = 1.0;
  let mut amplitude = 1.0;
  let mut max = 0.0;
  for _ in 0..octaves {
    max += amplitude;
    value += amplitude * perlin(seed, x * frequency, y * frequency);
    frequency *= lacunarity;
    amplitude *= gain;
  }
  value / max
}
