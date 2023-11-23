
use serde::Serialize;

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize)]
pub struct Geodata {
  pub ip: String,
  pub country: String,
  pub city: String,
  pub country_code: String,
  pub isp: String,
  pub region_name: String,
  pub lat: String, // .parse::<f64>().unwrap()
  pub lon: String, // .parse::<f64>().unwrap()
}
impl Geodata {
  pub fn new() -> Self {
    Self::default()
  }
}
