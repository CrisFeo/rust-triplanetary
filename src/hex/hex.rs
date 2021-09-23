use std::ops::{
  Add,
  Sub,
  Mul,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub struct Hex {
  pub q: i32,
  pub r: i32,
  pub s: i32,
}

#[derive(Copy, Clone, Debug)]
pub enum Path {
  One(Hex),
  Alt(Hex, Hex),
}

impl Hex {

  pub fn new(q: i32, r: i32, s: i32) -> Hex {
    Hex { q, r, s }
  }

  pub fn len(self) -> i32 {
    self.q.abs().max(self.r.abs()).max(self.s.abs())
  }

  pub fn direction(i: i32) -> Hex {
    let directions = [
      hex( 1, -1),
      hex( 1,  0),
      hex( 0,  1),
      hex(-1,  1),
      hex(-1,  0),
      hex( 0, -1),
    ];
    directions[(i % 6) as usize]
  }

  pub fn ring(self, radius: i32) -> Vec<Hex> {
    let mut results = vec![];
    add_ring(self, radius, &mut results);
    results
  }

  pub fn spiral(self, radius: i32) -> Vec<Hex> {
    let mut results = vec![];
    for k in 1..=radius {
      add_ring(self, k, &mut results);
    }
    results
  }

  pub fn line(self, other: Hex) -> Vec<Path> {
    use std::f32;
    let mut results = vec![];
    let n = (self - other).len();
    let step = 1. / (i32::max(n, 1) as f32);
    for i in 0..=n {
      let t = step * (i as f32);
      let qu = lerp(self.q as f32 + EPSILON, other.q as f32 + EPSILON, t).round();
      let ru = lerp(self.r as f32 - EPSILON, other.r as f32 - EPSILON, t).round();
      let u = hex(qu as i32, ru as i32);
      let qv = lerp(self.q as f32 - EPSILON, other.q as f32 - EPSILON, t).round();
      let rv = lerp(self.r as f32 + EPSILON, other.r as f32 + EPSILON, t).round();
      let v = hex(qv as i32, rv as i32);
      if u == v {
        results.push(Path::One(u));
      } else {
        results.push(Path::Alt(u, v));
      }
    }
    results
  }

}

const EPSILON : f32 = 1E-6;

fn lerp(a: f32, b: f32, t: f32) -> f32 {
  a + (b - a) * t
}

pub fn hex(q: i32, r: i32) -> Hex {
  Hex::new(q, r, -q - r)
}

fn add_ring(hex: Hex, radius: i32, results: &mut Vec<Hex>) {
  let mut current = hex + Hex::direction(4) * radius;
  for i in 0..6 {
    for _ in 0..radius {
      results.push(current);
      current = current + Hex::direction(i);
    }
  }
}

impl Add for Hex {
  type Output = Self;

  fn add(self, other: Self) -> Self {
    Self {
      q: self.q + other.q,
      r: self.r + other.r,
      s: self.s + other.s,
    }
  }
}

impl Add<i32> for Hex {
  type Output = Self;

  fn add(self, other: i32) -> Self {
    Self {
      q: self.q + other,
      r: self.r + other,
      s: self.s + other,
    }
  }
}

impl Add<Hex> for i32 {
  type Output = Hex;

  fn add(self, other: Hex) -> Hex {
    Hex {
      q: self + other.q,
      r: self + other.r,
      s: self + other.s,
    }
  }
}

impl Sub for Hex {
  type Output = Self;

  fn sub(self, other: Self) -> Self {
    Self {
      q: self.q - other.q,
      r: self.r - other.r,
      s: self.s - other.s,
    }
  }
}

impl Sub<i32> for Hex {
  type Output = Self;

  fn sub(self, other: i32) -> Self {
    Self {
      q: self.q - other,
      r: self.r - other,
      s: self.s - other,
    }
  }
}

impl Sub<Hex> for i32 {
  type Output = Hex;

  fn sub(self, other: Hex) -> Hex {
    Hex {
      q: self - other.q,
      r: self - other.r,
      s: self - other.s,
    }
  }
}

impl Mul for Hex {
  type Output = Self;

  fn mul(self, other: Self) -> Self {
    Self {
      q: self.q * other.q,
      r: self.r * other.r,
      s: self.s * other.s,
    }
  }
}

impl Mul<i32> for Hex {
  type Output = Self;

  fn mul(self, other: i32) -> Self {
    Self {
      q: self.q * other,
      r: self.r * other,
      s: self.s * other,
    }
  }
}

impl Mul<Hex> for i32 {
  type Output = Hex;

  fn mul(self, other: Hex) -> Hex {
    Hex {
      q: self * other.q,
      r: self * other.r,
      s: self * other.s,
    }
  }
}
