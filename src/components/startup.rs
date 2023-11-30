/// Startup Component
/// Acquires db connection
/// Sets up initial db
/// 
/// 
/// Holds the DB connection and handles queries.



use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{action::Action, config::key_event_to_string, themes, animations::Animation, migrations::schema, geofetcher};

use rand::prelude::*;

use rusqlite::{Connection, Result as ConnectionResult};
use tokio::sync::Mutex;
use std::sync::Arc;

use chrono::Utc;
use chrono;

use regex::Regex;

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
  }


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Init,
  Loading,
  Done,
  Confirmed,
}

#[derive(Default)]
pub struct Startup <'a>{
  pub show_help: bool,
  pub counter: usize,
  pub app_ticker: usize,
  pub render_ticker: usize,
  pub mode: Mode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub text: Vec<String>,
  pub last_events: Vec<KeyEvent>,
  pub num_ticks: usize,
  apptheme: themes::Theme,
  elapsed_frames: f64,
  points: Vec<(f64,f64,f64,f64)>,

  // db connection,
  dbconn: Option<Connection>,

  // Loading Ops
  log_messages: Vec<String>,


  // Animations
  anim_dotdotdot: Animation<&'a str>,

  // tmp
  last_ip: String,
  stored_geo: Vec<schema::IP>,

}

impl <'a> Startup <'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  fn set_items(mut self) -> Self {
    self.anim_dotdotdot = Animation::with_items(vec![".", "..", "..."]); // WHY THE FUCK IS THIS EMPTY ??????v bxdx,bd ,xd,v
    self
  }

  pub fn create_db(&mut self) {
    let dt = Utc::now();
    self.log_messages.push(format!("{}            init db", dt.to_string()));

    self.dbconn.as_ref().unwrap().execute(schema::CREATE_COUNTRY_DB_SQL, []).expect("Error setting up country db");

    self.dbconn.as_ref().unwrap().execute(schema::CREATE_CITY_DB_SQL, []).expect("Error setting up city db");

    self.dbconn.as_ref().unwrap().execute(schema::CREATE_REGION_DB_SQL, []).expect("Error setting up city db");

    self.dbconn.as_ref().unwrap().execute(schema::CREATE_ISP_DB_SQL, []).expect("Error setting up ISP db");

    self.dbconn.as_ref().unwrap().execute(schema::CREATE_IP_DB_SQL, []).expect("Error setting up IP db");

    let dt = Utc::now();
    self.log_messages.push(format!("{}            db ready", dt.to_string()));
  }

  pub fn set_rng_points(mut self) -> Self {
    let mut rng = rand::thread_rng();
    let num_lines: usize = rng.gen_range(0..20);
    let mut points: Vec<(f64,f64,f64,f64)> = vec![];
    for _ in 0..num_lines {
        let x: f64 = 0.;//rng.gen_range(-180.0..180.0);
        let y: f64 = 0.;//rng.gen_range(-90.0..90.0);
        let x2: f64 = rng.gen_range(-180.0..180.0);
        let y2: f64 = rng.gen_range(-90.0..90.0);
        points.push((x,y,x2,y2));
    }

    self.points = points;
    self
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }



  pub fn tick(&mut self) {
    log::info!("Tick");
    self.num_ticks += 1;
    self.anim_dotdotdot.next();
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    log::debug!("Render Tick");
    self.elapsed_frames += 1.;

    if self.elapsed_frames == 1. {
        let mut rng = rand::thread_rng();
        let x: f64 = 0.;//rng.gen_range(-180.0..180.0);
        let y: f64 = 0.;
        let x2: f64 = rng.gen_range(-180.0..180.0);
        let y2: f64 = rng.gen_range(-90.0..90.0);
        self.points.push((x,y,x2,y2));
        if self.points.len() > 20 {
            self.points = vec![];
        }
    }

    if self.elapsed_frames > 12. {
        self.elapsed_frames = 0.;
        if self.num_ticks > 12 {
            self.mode = Mode::Done;
            let _ = self.action_tx.clone().unwrap().send(Action::StartupDone);
        }
    }
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn add(&mut self, s: String) {
    self.text.push(s)
  }

}

impl Component for Startup <'_> {

  fn init(&mut self, area: Rect) -> Result<()> {
      
      self.action_tx.clone().unwrap().send(Action::StartupConnect).expect("Action::StartupConnect failed to send!");

      Ok(())
  }

  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Init => {Action::Render},
      Mode::Done => Action::StartupDone,
      _ =>  {self.input.handle_event(&crossterm::event::Event::Key(key));
      Action::Render},
    };
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {self.tick()},
      Action::Render => self.render_tick(),
      Action::StartupConnect => {
        let dt = Utc::now();

        self.log_messages.push(format!("{}            Connecting to db", dt.to_string()));

        let conn = Connection::open("iplogs.db")?;

        self.dbconn = Some(conn);
        self.create_db();
        
      },
      Action::IONotify(ref x) => {
        // got new line

        let re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
        let results: Vec<&str> = re
          .captures_iter(x)
          .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
          .collect();
        let cip: &str;
        // filtered for IP

        if !results.is_empty() {
          cip = results[0];
          // string contained an IPv4

          // check if is banned
          let output = std::process::Command::new("fail2ban-client")
            .arg("status")
            .arg("sshd")
            // Tell the OS to record the command's output
            .stdout(std::process::Stdio::piped())
            // execute the command, wait for it to complete, then capture the output
            .output()
            // Blow up if the OS was unable to start the program
            .unwrap();
  
          // extract the raw bytes that we captured and interpret them as a string
          let stdout = String::from_utf8(output.stdout).unwrap();
          let mut is_banned = false;

          if stdout.contains(cip) {
            is_banned = true;
          }


          //let mut is_in_list: bool = false;
          let conn = self.dbconn.as_ref().unwrap();
          //let conn2 = conn.clone();

          let maybe_data = schema::select_ip(conn, cip).unwrap_or_default().take().unwrap_or_default();

          if maybe_data == schema::IP::default() {
            // we have to fetch the data
            let timestamp = chrono::offset::Local::now();
  

            self.last_ip = String::from(cip);

            let req_ip = cip.to_string();
            let sender = self.action_tx.clone().unwrap();

            let handle = tokio::task::spawn(async move {
              // perform some work here...
              let geodat = geofetcher::fetch_geolocation(req_ip.as_str()).await.unwrap_or(serde_json::Value::default());

              let geoip = String::from(geodat.get("query").unwrap().as_str().unwrap());
              let geolat = geodat.get("lat").unwrap().as_number().unwrap().to_string();
              let geolon = geodat.get("lon").unwrap().as_number().unwrap().to_string();
              let geoisp = String::from(geodat.get("isp").unwrap().as_str().unwrap());
  
              let geocountry = String::from(geodat.get("country").unwrap().as_str().unwrap());
              let geocity = String::from(geodat.get("city").unwrap().as_str().unwrap());
              let geocountrycode = String::from(geodat.get("countryCode").unwrap().as_str().unwrap());
              let georegionname = String::from(geodat.get("regionName").unwrap().as_str().unwrap());
  
  
              let mut geodata: schema::IP = schema::IP::default();
              geodata.created_at = timestamp.to_string();
              geodata.ip = geoip;
              geodata.lat = geolat;
              geodata.lon = geolon;
              geodata.isp = geoisp;
              geodata.is_banned = is_banned;
              geodata.banned_times = match is_banned {false => 0, true => 1};
              geodata.country = geocountry;
              geodata.countrycode = geocountrycode;
              geodata.city = geocity;
              geodata.region = georegionname;
              geodata.warnings = 1;

              

            
              sender.send(Action::GotGeo(geodata)).unwrap_or_default();
            });
            
          }
          else {
            // data is stored
            self.action_tx.clone().unwrap().send(Action::GotGeo(maybe_data))?;  
          }


        }

      },
      Action::GotGeo(x) => {

        let conn = self.dbconn.as_ref().unwrap();
        let mut ip = schema::select_ip(conn, x.ip.as_str()).unwrap_or_default().unwrap_or_default();
        let mut ip_in_db: bool = true;

        if ip == schema::IP::default() {
          ip_in_db = false;
        }

        let mut isp: schema::ISP = schema::select_isp(conn, x.isp.as_str()).unwrap_or_default().unwrap_or_default();
        if isp == schema::ISP::default() {
          let _ = schema::insert_new_ISP(conn, x.isp.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1)).unwrap();
        }
        else {
          isp.warnings += 1;
          if !ip_in_db && x.is_banned {isp.banned += 1;}
          let _ = schema::insert_new_ISP(conn, isp.name.as_str(), Some(isp.banned), Some(isp.warnings)).unwrap();
        }

        let mut country = schema::select_country(conn, x.country.as_str()).unwrap_or_default().unwrap_or_default();
        if country == schema::Country::default() {
          let _ = schema::insert_new_country(conn, x.country.as_str(), Some(x.countrycode.as_str()), match x.is_banned {false => Some(0), true => Some(1)}, Some(1)).unwrap();
        }
        else {
          country.warnings += 1;
          if !ip_in_db && x.is_banned {country.banned += 1;}
          let _ = schema::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings)).unwrap();
        }

        let mut region = schema::select_region(conn, x.region.as_str()).unwrap_or_default().unwrap_or_default();
        if region == schema::Region::default() {
          let _ = schema::insert_new_region(conn, x.region.as_str(), x.country.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1)).unwrap();
        }
        else {
          region.warnings += 1;
          if !ip_in_db && x.is_banned {region.banned += 1;}
          let _ = schema::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings)).unwrap();
        }

        let mut city = schema::select_city(conn, x.city.as_str()).unwrap_or_default().unwrap_or_default();
        if city == schema::City::default() {
          let _ = schema::insert_new_city(conn, x.city.as_str(), x.country.as_str(), x.region.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1)).unwrap();
        }
        else {
          city.warnings += 1;
          if !ip_in_db && x.is_banned {city.banned += 1;}
          let _ = schema::insert_new_city(conn, city.name.as_str(), city.country.as_str(),city.region.as_str(), Some(city.banned), Some(city.warnings)).unwrap();
        }

        
        if !ip_in_db {
          let _ = schema::insert_new_IP(conn, 
            x.ip.as_str(), x.created_at.as_str(), 
            x.lon.as_str(), x.lat.as_str(), 
            x.isp.as_str(), x.city.as_str(), 
            Some(x.region.as_str()), x.country.as_str(),
            Some(x.countrycode.as_str()), x.banned_times, 
              x.is_banned, x.warnings).unwrap();
        }
        else {
          ip.warnings += 1;
          let _ = schema::insert_new_IP(conn,
            x.ip.as_str(), x.created_at.as_str(), 
            x.lon.as_str(), x.lat.as_str(), 
            x.isp.as_str(), x.city.as_str(), 
            Some(x.region.as_str()), x.country.as_str(),
            Some(x.countrycode.as_str()), x.banned_times, 
              x.is_banned, ip.warnings).unwrap();
        }
        //self.stored_geo.push(x.clone()); 
      }
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {


    match self.mode {
        Mode::Loading | Mode::Init => {





            let layout = Layout::default().constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref()).split(rect);

            let canvas = canvas::Canvas::default()
            .background_color(self.apptheme.colors.default_background)
            .block(Block::default().borders(Borders::ALL).title("World").bg(self.apptheme.colors.default_background))
            .marker(Marker::Braille)
            .paint( |ctx| {

    
                ctx.draw(&canvas::Map {
                    color: self.apptheme.colors.default_map_color,
                    resolution: canvas::MapResolution::High,
                });

                for point in self.points.iter() {

                    let direction = (point.0 - point.2, point.1 - point.3);

                    ctx.draw(&canvas::Line {
                        x1: point.0,
                        y1: point.1,
                        x2: point.2,
                        y2: point.3,
                        color:self.apptheme.colors.accent_dblue,
                      }); 
          
                      ctx.draw(&canvas::Line {
                        x1: point.2 + direction.0 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        y1: point.3 + direction.1 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        x2: point.2,
                        y2: point.3,
                        color: self.apptheme.colors.accent_blue,
                      });                    

                } 


                ctx.draw(&canvas::Circle {
                    x: 0., // lon
                    y: 0., // lat
                    radius:  self.render_ticker as f64,
                    color: self.apptheme.colors.accent_orange,
                  });

            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);         

            
            let frame_idx = self.anim_dotdotdot.state.selected().unwrap_or_default();
            let selected_frame = self.anim_dotdotdot.keyframes[frame_idx];

            let mut loglines: Vec<Line> = vec![];
            loglines.push(Line::from(format!("Render Ticker: {}", self.render_ticker)));
            loglines.push(Line::from(format!("App Ticker: {}", self.app_ticker)));

            let num_msgs = self.log_messages.len();
            for i in 0..num_msgs {
              
              if i == num_msgs - 1 {
                loglines.push(Line::from(format!("{}{}", self.log_messages[i], selected_frame)));
              } else {
                loglines.push(Line::from(format!("{}", self.log_messages[i])));
              }
              
            }


/*             let mut text: Vec<Line> = self.text.clone().iter().map(|l| Line::from(l.clone())).collect();
            text.insert(0, "".into());
            text.insert(0, "Loading".into());
            text.insert(0, format!("{}", selected_frame).into());
            text.insert(0, "".into());
            text.insert(0, format!("Render Ticker: {}", self.render_ticker).into());
            text.insert(0, format!("App Ticker: {}", self.app_ticker).into());
            text.insert(0, format!("Counter: {}", self.counter).into());
            text.insert(0, "".into());
            text.insert(
            0,
            Line::from(vec![
                "Press ".into(),
                Span::styled("j", Style::default().fg(Color::Red)),
                " or ".into(),
                Span::styled("k", Style::default().fg(Color::Red)),
                " to ".into(),
                Span::styled("increment", Style::default().fg(Color::Yellow)),
                " or ".into(),
                Span::styled("decrement", Style::default().fg(Color::Yellow)),
                ".".into(),
            ]),
            ); */
            //text.insert(0, "".into());

            f.render_widget(
                Paragraph::new(loglines)
                  .block(
                    Block::default()
                      .title("Setting up")
                      .title_alignment(Alignment::Center)
                      .borders(Borders::ALL)
                      .border_style(self.apptheme.border_style)
                      .border_type(BorderType::Rounded),
                  )
                  .style(Style::default().fg(Color::White).bg(self.apptheme.colors.lblack)) //self.apptheme.colors.accent_blue
                  .alignment(Alignment::Left),
                layout[1],
              );

            f.render_widget(canvas, layout[0]);


        },
        _ => {},
    }

    Ok(())
  }
}