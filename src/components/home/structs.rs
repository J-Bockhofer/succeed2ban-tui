use tokio::time;
use crate::database::schema::ip::IP;
use ratatui::prelude::Style;

#[derive(Default, Clone)]
pub struct StyledLine {
  pub words: Vec<(String, Style)>,
}

#[derive(Clone, PartialEq)]
pub struct PointData {
  pub ip: String,
  pub lon: f64,
  pub lat: f64,
  pub dir_home_lon: f64,
  pub dir_home_lat: f64,
  pub start_time: tokio::time::Instant,
  pub is_alive: bool,
}

impl PointData {
  pub fn new(ip: String, lon:f64, lat:f64, dir_lon: f64, dir_lat: f64)-> Self {
    PointData { ip, lon, lat, dir_home_lon: dir_lon, dir_home_lat: dir_lat, start_time: tokio::time::Instant::now(), is_alive: true }
  }
  pub fn decay_point(&mut self, decaytime: tokio::time::Duration) {
    if self.start_time.elapsed() > decaytime {
      self.is_alive = false;
    } 
    else {
      self.is_alive = true;
    }
  }

  pub fn refresh(&mut self) {
    let timenow = tokio::time::Instant::now();
    self.is_alive = true;
    self.start_time = timenow;
  }
  
}

impl Default for PointData {
   fn default() -> Self {
      PointData::new(String::default(), f64::default(), f64::default(), f64::default(), f64::default())
  }
}

//iplist: StatefulList<(String, schema::IP, String)>,
#[allow(non_snake_case)]
#[derive(Clone, Default)]
pub struct IPListItem {
  pub IP: IP,
  pub username: String,
  pub pointdata: PointData,
}
#[allow(non_snake_case)]
impl IPListItem {
  pub fn new(IP:IP, username:String, pointdata: PointData)-> Self {
    IPListItem { IP, username, pointdata}
  }
}
