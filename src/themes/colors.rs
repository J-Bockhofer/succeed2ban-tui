
use ratatui::prelude::Color;
use std::{str::FromStr, fmt::Error};

// DefaultDark Colors
pub const BACKGROUND: Color = Color::Rgb(37,37,38);
pub const ACCENT_BLUE: Color = Color::Rgb(136,171,246);
pub const ACCENT_DBLUE: Color = Color::LightBlue;
pub const ACCENT_ORANGE: Color = Color::Rgb(241,185,65);
pub const ACCENT_LORANGE: Color = Color::Rgb(246,217,152);

pub const ACCENT_WRED: Color = Color::Rgb(128,0,64);
pub const ACCENT_LPINK: Color = Color::Rgb(248,182,203);
pub const ACCENT_LIME: Color = Color::Rgb(203,248,182);

pub const LBLACK: Color = Color::Rgb(32, 32, 32);
pub const DDBLUE: Color = Color::Rgb(32,38,72);
//(32,55,114);
pub const EMPTY: Color = Color::Rgb(0,0,0);

// Nice Fat Blue
pub const BBLUE: Color = Color::Rgb(36,71,146);

// DefaultBright Colors
pub const BRIGHT_BACKGROUND: Color = Color::Rgb(255,249,236);
pub const LIGHT_BACKGROUND: Color = Color::Rgb(250,244,231);
pub const LIGHT_BACKGROUND_2: Color = Color::Rgb(239,232,219);
pub const LIGHT_LIME: Color = Color::Rgb(200,235,119);
pub const DORANGE: Color = Color::Rgb(168,129,45);
pub const DDPURP: Color = Color::Rgb(75,55,93);
pub const DPURP: Color = Color::Rgb(106,50,159);
pub const LPURP: Color = Color::Rgb(150,111,187);
// Color = Color::Rgb(150,111,187)
//pub const LIGHT_MAP: Color = Color::Rgb(250,244,231);


#[derive(Default, Clone)]
pub struct ColorRGB {
  pub color: Color,
  pub r: u8,
  pub g: u8,
  pub b: u8,
}

impl ColorRGB {
  pub fn new(r: u8, g: u8, b: u8) -> Self {
    let color = Color::Rgb(r, g, b);
    ColorRGB { color, r, g, b }
  }
  /// Creates a shaded variant of the passed color.
  /// frac is clamped between -1 and 1 -> -100% brightness to +100% brightness.
  pub fn shade(&self, frac: f32) -> Color {
    if frac < 0. {
      // darken
      let r = darken_channel(self.r, frac);
      let g = darken_channel(self.g, frac);
      let b = darken_channel(self.b, frac);
      Color::Rgb(r, g, b)
    } else {
      // brighten
      let r = brighten_channel(self.r, frac);
      let g = brighten_channel(self.g, frac);
      let b = brighten_channel(self.b, frac);
      Color::Rgb(r, g, b)
    }
  }

  pub fn from_hex(s: &str) -> Result<Self, Error> { 
    match s
    .to_lowercase()
    .replace([' ', '-', '_'], "")
    .as_ref()
    {
      "white" => {Ok(Self::new(255,255,255))},
      "lightblue" => {Ok(Self::new(48,48,240))},
      _ => { if let (Ok(r), Ok(g), Ok(b)) = {
        if !s.starts_with('#') || s.len() != 7 {
            return Err(Error);
        }
          (
              u8::from_str_radix(&s[1..3], 16),
              u8::from_str_radix(&s[3..5], 16),
              u8::from_str_radix(&s[5..7], 16),
          )
        } {
            Ok(Self::new(r, g, b))
        } else {
            return Err(Error);
        }

      }
    }

  }

  pub fn from_color(color: Color) -> Result<Self, Error> {
    Self::from_hex(&color.to_string())
  }

  pub fn flip_rgb(&self) -> Color {
    let r = u8::MAX - self.r;
    let g = u8::MAX - self.g;
    let b = u8::MAX - self.b;
    Color::Rgb(r, g, b)
  }

}


pub fn brighten_channel(x: u8, inc: f32) -> u8 {
  let mut inc = inc;
  if inc <= 0. {inc = inc.abs();};
  if inc >= 1. {return u8::MAX;};
  let mut _x = u8::MAX - x;
  let mut __x: f32 = _x as f32;
  __x = __x * inc;
  _x = __x as u8;
  x.saturating_add(_x)
}

pub fn darken_channel(x: u8, dec: f32) -> u8 {
  let mut dec = dec;
  if dec <= 0. { dec = dec.abs();};
  if dec >= 1. { return u8::MIN;};
  let mut _x: f32 = x as f32;
  _x = _x * dec;
  let __x = _x as u8;
  x.saturating_sub(__x)
}
