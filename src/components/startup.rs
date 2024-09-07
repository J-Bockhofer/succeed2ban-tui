use std::process::Output;
/// Startup Component
/// Acquires db connection
/// Sets up initial db
/// 
/// 
/// Holds the DB connection and handles queries.
mod actions;
mod db_actions;
mod f2b_actions;

use std::sync::OnceLock;


use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use lazy_static::__Deref;
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::gen_structs::StatefulList;
use crate::tasks::{IOMessage, IOProducer};
use crate::themes::ThemeContainer;
use crate::{action::Action, config::key_event_to_string, config::Config, themes, animations::Animation, database::schema, geofetcher};
use crate::database::schema::{message, isp, city, region, country, ip};



use rand::prelude::*;

use rusqlite::{Connection, Result as ConnectionResult};
use tokio::sync::Mutex;
use std::sync::Arc;

use chrono::Utc;
use chrono;

use regex::Regex;

pub fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
  }



#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Init,
  Loading,
  Done,
  Completed,
  Confirmed,
}

#[derive(Default)]
pub struct Startup <'a>{
  config: Config,
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
  countdown_to_start: usize,

  points: Vec<(f64,f64,f64,f64)>,

  // db connection,
  dbconn: Option<Connection>,


  log_messages: Vec<String>,


  // Animations
  anim_dotdotdot: Animation<&'a str>,
  anim_charsoup: Animation<&'a str>,

  // tmp
  last_ip: String,
  fetching_ips:Vec<String>,
  //stored_geo: Vec<ip::IP>,

  // startup line
  startup_lines: Vec<&'a str>,

  available_themes: StatefulList<ThemeContainer>,


}

impl <'a> Startup <'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  fn set_items(mut self) -> Self {
    self.anim_dotdotdot = Animation::with_items(vec![".", "..", "..."]); 
    self.anim_charsoup = Animation::with_items(vec!["dcc&ßm-)44sas/a.sc&%cßd%acb8ß0bj
  )1d.yß.1ybd4e.)-j6155dßße0#4(-6&
  m/.,5ess#05%-ssâ3/jej-cs6s.e.s-s
  d-s)38&m-a/s-0s/bjbd6%ssmb0-b(&(
  b%3(bcjc4(a0/3c0c1(4-,3//eß,8ß/y", "%d.%%%d#(,bâ-s&)3y3ac5y#64-&-/s,
    dßsyßâ#c&#6mdßbj6m6&65(cs/sy1yß%
    41,..,j08&#6,68&yß-s1d4âs6b#e,a&
    8.yy36s,y56c(5d-c8.&/%&58s35s,s6
    /)-.5#&,ß01my&&sce033ß8-)ma/cc6s", "sßâßyc&-/â,65.ma/#5eâ/ya4/&dc&m
    .10ems.css4(m33mßay84yj.cße4yd&
    &e-8#36#y,yse,a0syy(/ßm-563ßc5y1
    5#ccs&-e(â-1ß113ßsjd-j-.,a#j3(c
    s351-ac3b)c#b.0(b,)a5085d4,s0c&d",
    "â6c#.8(ms/)&381câd6â%1b,sâßcde1s
    eß13âsß3s#8j.ca&5ß%s/#âj&a.md%ß-
    ßeys)sß4â5s63ßsd%31,88c4ß-b.b%5c
    .#)344aese#s&d/%â5sa,c)./bs4cs-j
    ,dsme(jâ5(6%s5.bc,eb-36ycce5e,5d",
    "/)c%#mgfc.-m,-mykm-hcshyy##&4y))
    (1c/ß/k4,./6%ch.ßmg7-429hdfk%c)/
    dyksh-%,ym.âc1g)dh-âs/yd%l%.4c,7
    .l0#-sh9k/6kl/l,a.,cyâ00m2.%hl-,
    sâs-ß1-%h(.yyßhaamyc2ßk7l)c.gcßf",
    "gd0)â6%9c.d7170âhdk-4/6a0-#kdylh
    c7k7ß#s1)2(ß(.h92â2g2gsg#46c(gh#
    aß6,algâds&/)0,y(-mâk&d2lhcß(-(m
    -#4.f,f&))â07c-9,l)c&#4g,/c%)â%h
    lf0ß4dl09f7/mms#.d2hmf44gf-c-10m"]);
    self.startup_lines =  vec![
      "Eating logfiles",
     "Setting up stderr",
      "Deciphering binaries",
       "Attempting to retaliate",
        "Starting the engines", 
        "Processing db",
         "Calling home",
          "Tracing routes",];
    self.countdown_to_start = 10;
    self.fetching_ips = vec![];
    self.available_themes.items = themes::Themes::default().theme_collection;
    self
  }

  pub fn create_db(&mut self) {
    let dt = Utc::now();
    self.log_messages.push(format!("{}            init db", dt.to_string()));

    schema::create_tables(self.dbconn.as_ref().unwrap()).expect("Error setting up tables");

    let dt = Utc::now();
    self.log_messages.push(format!("{}            db ready", dt.to_string()));


  }

  pub fn get_initial_stats(&mut self) {
    let tx = self.action_tx.clone().unwrap();
    let dt = Utc::now();
    self.log_messages.push(format!("{}            Fetching stats", dt.to_string()));
    tokio::spawn(async move {
      tx.send(Action::StatsGetCountries).expect("Failed to get Countries on Startup");    
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
      tx.send(Action::StatsGetCities).expect("Failed to get Cities on Startup");
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
      tx.send(Action::StatsGetRegions).expect("Failed to get Regions on Startup");   
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
      tx.send(Action::StatsGetISPs).expect("Failed to get ISPs on Startup");  
      tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;    
    });
    let rndline = self.get_rnd_start_msg(self.startup_lines.clone());
    self.log_messages.push(format!("{}            {}", dt.to_string(), rndline));
  }

  pub fn get_rnd_start_msg(&self, lines: Vec<&str>) -> String {
    let mut rng = rand::thread_rng();
    let line: usize = rng.gen_range(0..lines.len());
    lines[line].to_string()
  }

  fn set_rng_points(mut self) -> Self {
    let mut rng = rand::thread_rng();
    let num_lines: usize = rng.gen_range(0..20);
    let mut points: Vec<(f64,f64,f64,f64)> = vec![];
    for _ in 0..num_lines {
        let x: f64 = 0.;
        let y: f64 = 0.;
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

  pub fn f2b_check_banned(&self, cip: &str) -> bool {
    let output = std::process::Command::new("fail2ban-client")
      .arg("status")
      .arg("sshd")
      // Tell the OS to record the command's output
      .stdout(std::process::Stdio::piped())
      // execute the command, wait for it to complete, then capture the output
      .output();
      // Blow up if the OS was unable to start the program // No pls dont
    let stdout: String;
    if output.is_ok() {
      stdout = String::from_utf8(output.unwrap().stdout).unwrap();
    }  else {
      let fetchmsg = format!("   Failed to run fail2ban client for IP look-up");
      self.action_tx.clone().unwrap().send(Action::InternalLog(fetchmsg)).expect("CRITICAL: Fail to run message failed to send, yes... really... EpicFail");
      stdout = String::from("");
    }  
    // extract the raw bytes that we captured and interpret them as a string        
    if stdout.contains(cip) {
      true
    }
    else {
      false
    }
  }


  pub fn set_theme(&mut self) {
    let theme_idx = self.available_themes.state.selected();
    if theme_idx.is_some() {
      let theme_idx = theme_idx.unwrap();
      let tx = self.action_tx.clone().unwrap();
      let selected_theme = self.available_themes.items[theme_idx].clone();
      self.apptheme = selected_theme.theme;

      tx.send(Action::SelectTheme(selected_theme.name)).expect("Error sending Theme Change");
    }

  }

  pub fn add_thyme(&mut self) {
    self.countdown_to_start = self.countdown_to_start.saturating_add(2);
  }

  pub fn tick(&mut self) {
    self.num_ticks += 1;
    self.anim_dotdotdot.next();
    self.countdown_to_start = self.countdown_to_start.saturating_sub(1);

    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    self.elapsed_frames += 1.;
    self.anim_charsoup.next();
    
    if self.elapsed_frames == 1. {
        let mut rng = rand::thread_rng();
        let x: f64 = 0.;
        let y: f64 = 0.;
        let x2: f64 = rng.gen_range(-180.0..180.0);
        let y2: f64 = rng.gen_range(-90.0..90.0);
        self.points.push((x,y,x2,y2));
        if self.points.len() > 20 {
            self.points = vec![];
        }
    }
    if self.mode == Mode::Done && self.countdown_to_start == 0 {
      if self.elapsed_frames > 12.  { // sync load with anim
        //self.set_theme();
        let _ = self.action_tx.clone().unwrap().send(Action::StartupDone);
      }      
    }

    if self.elapsed_frames > 12. {
        self.elapsed_frames = 0.;
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
  
  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Init => {Action::Render},
      Mode::Done => {
        match key.code {
          KeyCode::Up => {self.available_themes.previous(); self.add_thyme();},
          KeyCode::Down => {self.available_themes.next(); self.add_thyme();},
          KeyCode::Enter => {self.set_theme(); self.add_thyme();},
          KeyCode::Esc => {return Ok(Some(Action::Quit))},
          KeyCode::BackTab => {self.countdown_to_start = 1;},
          KeyCode::Tab => {self.countdown_to_start = 1;}
          _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
        }
        
        Action::Blank},
      _ =>  {self.input.handle_event(&crossterm::event::Event::Key(key));
      Action::Blank},
    };
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    let tx = self.action_tx.clone().unwrap();
    match action {
      
      Action::Tick => {self.tick()},
      Action::Render => self.render_tick(),
      Action::StartupDone => {self.mode = Mode::Completed;
        let tx = self.action_tx.clone().unwrap();
        let fetchmsg = format!(" ✔ Startup Complete");
        tx.send(Action::InternalLog(fetchmsg)).expect("Fetchlog message failed to send");
      }
      Action::StartupConnect => {
        self.connect()?;        
      },
      Action::IONotify(iomsg) => {
        self.io_notify(iomsg)?;
      },
      Action::GotGeo(x, iomsg, from_db) => {
        self.got_geo(x, iomsg, from_db)?;
        //self.stored_geo.push(x.clone()); 
      },
      Action::SubmitQuery(querystr) => {
        let conn = self.dbconn.as_ref().unwrap();
        let tx = self.action_tx.clone().unwrap();
        db_actions::process_query(conn, querystr, tx);
      },
      Action::StatsGetCountries => {
        let conn = self.dbconn.as_ref().unwrap();
        let countries = country::get_all_countries(conn).unwrap_or(vec![]);
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
          let conn = Connection::open("iplogs.db").expect("Async thread DB connection failed");
          for country in countries {
            std::thread::sleep(std::time::Duration::from_millis(10)); // Debounce
            let timestamps = message::get_message_timestamps_by_country(&conn, &country.name).unwrap_or(vec![]);
            tx.send(Action::StatsGotCountry(country, timestamps)).expect("Failed to send Country to Stats");
         }
        });

      },
      Action::StatsGetRegions => {
        let conn = self.dbconn.as_ref().unwrap();
        let regions = region::get_all_regions(conn).unwrap_or(vec![]);
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
          let conn = Connection::open("iplogs.db").expect("Async thread DB connection failed");
          for region in regions {
            std::thread::sleep(std::time::Duration::from_millis(10)); // Debounce
            let timestamps = message::get_message_timestamps_by_region(&conn, &region.name).unwrap_or(vec![]);
            tx.send(Action::StatsGotRegion(region, timestamps)).expect("Failed to send Region to Stats");
         }
        });

      },
      Action::StatsGetISPs => {
        let conn = self.dbconn.as_ref().unwrap();
        let isps = isp::get_all_isps(conn).unwrap_or(vec![]);
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
          let conn = Connection::open("iplogs.db").expect("Async thread DB connection failed");
          for isp in isps {
            std::thread::sleep(std::time::Duration::from_millis(10)); // Debounce
            let timestamps = message::get_message_timestamps_by_isp(&conn, &isp.name).unwrap_or(vec![]);
            tx.send(Action::StatsGotISP(isp, timestamps)).expect("Failed to send ISP to Stats");
         }
        });
      },
      Action::StatsGetCities => {
        let conn = self.dbconn.as_ref().unwrap();
        let cities = city::get_all_cities(conn).unwrap_or(vec![]);
        let tx = self.action_tx.clone().unwrap();
        tokio::spawn(async move {
          let conn = Connection::open("iplogs.db").expect("Async thread DB connection failed");
          for city in cities {
            std::thread::sleep(std::time::Duration::from_millis(10)); // Debounce
            let timestamps = message::get_message_timestamps_by_city(&conn, &city.name).unwrap_or(vec![]);
            tx.send(Action::StatsGotCity(city, timestamps)).expect("Failed to send City to Stats");
         }
        });
      },

      Action::StatsBlockCountry(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let country = country::select_country(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = country::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings), true).unwrap();
        let fetchmsg = format!(" {} Blocked Country: {}", self.apptheme.symbols.block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block Country message failed to send");
      },
      Action::StatsUnblockCountry(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let country = country::select_country(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = country::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked Country: {}", self.apptheme.symbols.unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock Country message failed to send");
      },      
      Action::StatsBlockRegion(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let region = region::select_region(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), true).unwrap();
        let fetchmsg = format!(" {} Blocked Region: {}", self.apptheme.symbols.block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block Region message failed to send");
      },
      Action::StatsUnblockRegion(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let region = region::select_region(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked Region: {}", self.apptheme.symbols.unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock Region message failed to send");
      }, 
      Action::StatsBlockCity(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let city = city::select_city(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(), city.region.as_str(),Some(city.banned), Some(city.warnings), true).unwrap();
        let fetchmsg = format!(" {} Blocked City: {}", self.apptheme.symbols.block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block City message failed to send");
      },
      Action::StatsUnblockCity(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let city = city::select_city(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(), city.region.as_str(),Some(city.banned), Some(city.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked City: {}", self.apptheme.symbols.unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock City message failed to send");
      },    
      Action::StatsBlockISP(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let isp = isp::select_isp(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = isp::insert_new_ISP(conn, isp.name.as_str(),Some(isp.banned), Some(isp.warnings),isp.country.as_str(), true).unwrap();
        let fetchmsg = format!(" {} Blocked ISP: {}", self.apptheme.symbols.unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block ISP message failed to send");
      },
      Action::StatsUnblockISP(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let isp = isp::select_isp(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = isp::insert_new_ISP(conn, isp.name.as_str(), Some(isp.banned), Some(isp.warnings),isp.country.as_str(), false).unwrap();
        let fetchmsg = format!(" {} Unblocked ISP: {}", self.apptheme.symbols.unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock ISP message failed to send");
      }, 
      Action::StatsGetIP(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        let ipdata = ip::select_ip(conn, x.as_str()).unwrap_or_default().take().unwrap_or_default();
        if ipdata != ip::IP::default() {
          let tx = self.action_tx.clone().unwrap();
          tx.send(Action::StatsGotIP(ipdata)).expect("Failed to send IP data back to Stats");
        }
      },

      Action::BanIP(x) => {
        let cip = x.clone();
        let symb = self.apptheme.symbols.ban.clone();
        let _symb = self.apptheme.symbols.ban.clone();
        if !x.is_banned {

          let besure = self.f2b_check_banned(&x.ip);
          
          if !besure {        
            let tx = self.action_tx.clone().unwrap();
            f2b_actions::send_ban(x.clone(), symb, tx);
          } else {
            let blockmsg = format!(" {} IP already banned {}", _symb, &cip.ip);
            tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");   
            tx.send(Action::Banned(true)).expect("Failed to Ban ...");
          }
          tx.send(Action::Banned(true)).expect("Failed to Ban ...");
        } else {
          let blockmsg = format!(" {} IP already banned {}", _symb, &cip.ip);
          tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");  
          tx.send(Action::Banned(true)).expect("Failed to Ban ...");
        }
      },
      Action::UnbanIP(x) => {
        let cip = x.clone();
        let besure: bool;
        if !x.is_banned {
          besure = self.f2b_check_banned(&x.ip);
        } else {
          // x is banned
          besure = true;
        }
        if besure {
          let tx = self.action_tx.clone().unwrap();
          let symb = self.apptheme.symbols.unblock.clone();
          f2b_actions::send_unban(x.clone(), symb, tx);
        } else {
          let blockmsg = format!(" ! IP is not banned {}", &cip.ip);
          tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");   
        }
      },      

      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {

    match self.mode {
        Mode::Loading | Mode::Init | Mode::Done => {

            let layout = Layout::default().constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref()).split(rect);
            let sublayout = Layout::default().constraints([Constraint::Percentage(70), Constraint::Percentage(30)].as_ref()).direction(Direction::Horizontal).split(layout[1]);

            let canvas = canvas::Canvas::default()
            .background_color(self.apptheme.colors_app.background_mid.color)
            .block(Block::default().borders(Borders::ALL).title("").bg(self.apptheme.colors_app.background_mid.color))
            .marker(Marker::Braille)
            .paint( |ctx| {
   
                ctx.draw(&canvas::Map {
                    color: self.apptheme.colors_app.map_color.color,
                    resolution: canvas::MapResolution::High,
                });

                for point in self.points.iter() {

                    let direction = (point.0 - point.2, point.1 - point.3);

                    ctx.draw(&canvas::Line {
                        x1: point.0,
                        y1: point.1,
                        x2: point.2,
                        y2: point.3,
                        color:self.apptheme.colors_app.accent_color_b_mid.color,
                      }); 
          
                      ctx.draw(&canvas::Line {
                        x1: point.2 + direction.0 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        y1: point.3 + direction.1 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        x2: point.2,
                        y2: point.3,
                        color: self.apptheme.colors_app.accent_color_b_bright.color,
                      });                    

                } 


                ctx.draw(&canvas::Circle {
                    x: 0., // lon
                    y: 0., // lat
                    radius:  self.elapsed_frames,
                    color: self.apptheme.colors_app.accent_color_a.color,
                  });

            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);         

            let frame_idx = self.anim_charsoup.state.selected().unwrap_or_default();
            let selected_soup = self.anim_charsoup.keyframes[frame_idx];
            let chars: Vec<char> = selected_soup.chars().collect();

            let mut rng = rand::thread_rng();
            //let num_lines: f32 = rng.gen_range(-1..1.);
            let step: rand::distributions::Uniform<f64>;
            if self.apptheme.is_light {
              step = rand::distributions::Uniform::new(0., 1.);
            } else {
              step = rand::distributions::Uniform::new(-1., 0.);
            }
            
            

            let vecspan: Vec<Span> = chars.into_iter().map(|char|{
              let choice = step.sample(&mut rng) as f32;
              let color = self.apptheme.colors_app.text_color.shade(choice);
              let char = format!("{}",char);
              Span::styled(char, Style::default().fg(color))
            }).collect();

            //let soupline = Line::from(vecspan);
            
            let frame_idx = self.anim_dotdotdot.state.selected().unwrap_or_default();
            let selected_frame = self.anim_dotdotdot.keyframes[frame_idx];

            let mut loglines: Vec<Line> = vec![];
            loglines.push(Line::from(format!("Countdown to start: {}", self.countdown_to_start)));
            //loglines.push(Line::from(Span::styled(format!("          --             "), self.apptheme.styles_app.default_style.bg(self.apptheme.colors_app.background_brightest.color))));
            loglines.push(Line::from(format!("")));
            let num_msgs = self.log_messages.len();
            for i in 0..num_msgs {
              
              if i == num_msgs - 1 {
                loglines.push(Line::from(format!("{}{}", self.log_messages[i], selected_frame)));
                //loglines.push(Line::from(format!("{}", selected_soup)));
                loglines.push(Line::from(vecspan.clone()));
              } else {
                loglines.push(Line::from(format!("{}", self.log_messages[i])));
              }
              
            }


            // create list of themes
            let av_themes: Vec<ListItem> = self
            .available_themes
            .items
            .iter()
            .map(|i| {
                let line = Line::from(i.name.clone());
                ListItem::new(line).style(Style::default().fg(self.apptheme.colors_app.text_color.color))
            })
            .collect();
        
            // Create a List from all list items and highlight the currently selected one
            let themeslist = List::new(av_themes)
                .bg(self.apptheme.colors_app.background_darkest.color)
                .block(Block::default()
                .borders(Borders::ALL)
                .border_style( self.apptheme.styles_app.active_border_style)
                .title("Themes"))
                .highlight_style(self.apptheme.styles_app.highlight_item_style)
                .highlight_symbol(">> ");

            f.render_stateful_widget(themeslist, sublayout[1], &mut self.available_themes.state);

            f.render_widget(
                Paragraph::new(loglines)
                  .block(
                    Block::default()
                      .title("Setting up")
                      .title_alignment(Alignment::Center)
                      .borders(Borders::ALL)
                      .border_style(self.apptheme.styles_app.border_style)
                      .border_type(BorderType::Rounded),
                  )
                  .style(self.apptheme.styles_app.default_style).bg(self.apptheme.colors_app.background_darkest.color) //self.apptheme.colors.accent_blue
                  .alignment(Alignment::Left),
                sublayout[0],
              );

            f.render_widget(canvas, layout[0]);


        },
        _ => {},
    }

    Ok(())
  }
}