use ratatui::prelude::Color;
use std::fmt::Error;

#[derive(Default)]
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
      "white" => {Ok(Self::new(0,0,0))},
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

pub fn main() {
  let color = ColorRGB::new(248, 113, 113);
  //let color2 = ColorRGB::new(239, 68, 68);

  println!("{}", Color::White.to_string());
  let colora: Color = Color::Rgb(0, 88, 155);
  //;
  let color3 = ColorRGB::from_hex(&Color::White.to_string());

  assert_eq!(color3.unwrap().color.to_string(), "#000000".to_string());

  //                                       9     55   55
  //                                      16     187  187
  //    increase from ceiling            56%    29%

  let _r = 255 - color.r;
  let _g = 255 - color.g;
  let _b = 255 - color.b;

  let increase: f32 = 7. / 8.;

  // darken is not proportion and just mult -> rs = r * increase

  let __r: f32 = _r as f32;
  let __r = __r * increase;
  let _r = __r as u8;

  let __g: f32 = _g as f32;
  let __g = __g * increase;
  let _g = __g as u8;

  let __b: f32 = _b as f32;
  let __b = __b * increase;
  let _b = __b as u8;

  let new_r = color.r.saturating_add(_r);
  let new_g = color.g.saturating_add(_g);
  let new_b = color.b.saturating_add(_b);

  //let color_new = ColorRGB::new(new_r, new_g, new_b);

  println!("{},{},{}", new_r, new_g, new_b);
  println!("{}", colora.to_string());
}
