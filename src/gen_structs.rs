
use serde::Serialize;
use ratatui::{prelude::*, widgets::*};


#[derive(Default)]
pub struct StatefulList<T> {
  pub state: ListState,
  pub items: Vec<T>,
}

impl<T> StatefulList<T> {
  pub fn with_items(items: Vec<T>) -> StatefulList<T> {
      StatefulList {
          state: ListState::default(),
          items,
      }
  }

  pub fn next(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i >= self.items.len() - 1 {
                  0
              } else {
                  i + 1
              }
          }
          None => 0,
      };
      //println!("next Item: {i}");
      self.state.select(Some(i));
  }

  pub fn previous(&mut self) {
      let i = match self.state.selected() {
          Some(i) => {
              if i == 0 {
                  self.items.len() - 1
              } else {
                  i - 1
              }
          }
          None => 0,
      };
      self.state.select(Some(i));
  }

  pub fn unselect(&mut self) {
      self.state.select(None);
  }

  pub fn trim_to_length(&mut self, max_length: usize) {
    while self.items.len() > max_length {
        self.items.remove(0);
    }
  }
}



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
