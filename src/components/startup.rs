use std::process::Output;
/// Startup Component
/// Acquires db connection
/// Sets up initial db
/// 
/// 
/// Holds the DB connection and handles queries.
mod actions;

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
use crate::{action::Action, config::key_event_to_string, config::Config, themes, animations::Animation, migrations::schema, geofetcher};
use crate::migrations::schema::{message, isp, city, region, country, ip};



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
        // got new line
        let prod: IOProducer;
        let catmsg = match iomsg.clone() {
          IOMessage::SingleLine(x, p) => {
            prod = p;
            x},
          IOMessage::MultiLine(vx, p) => {
            prod = p;
            vx.join(" ")
          },
        };
        let re = self.apptheme.ipregex.clone();

        let results: Vec<&str> = re
          .captures_iter(&catmsg)
          .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
          .collect();

        if results.is_empty() {
          // results were empty, might happen if journalctl sends error message -> in that case just insert the last ip into the message and try again
          if !self.last_ip.is_empty() {
            let msg = crate::tasks::IOMessage::SingleLine(format!("{} for {}", catmsg, self.last_ip), prod);
            self.action_tx.clone().unwrap().send(Action::IONotify(msg))?;
          }
          return Ok(None)
        }

        let cip: &str;
        // filtered for IP
        cip = results[0];
        // string contained an IPv4
        let mut is_banned = false;
        if catmsg.contains("Ban") {
          is_banned = true;
        }

        if self.last_ip != String::from(cip) {
          std::thread::sleep(std::time::Duration::from_millis(100));
          // check if is banned
          is_banned = self.f2b_check_banned(cip);
        };

        let conn = self.dbconn.as_ref().unwrap();

        let mut maybe_data = ip::select_ip(conn, cip).unwrap_or_default().take().unwrap_or_default();
        self.last_ip = String::from(cip);
      
        if maybe_data == ip::IP::default() {
          // we have to fetch the data
          let sender = self.action_tx.clone().unwrap();
          self.fetching_ips.push(cip.to_string());
          actions::fetch_geolocation_and_report(cip.to_string(), is_banned.clone(), iomsg.clone(), sender);
        }
        else {
          // data is stored
          maybe_data.is_banned = is_banned;
          self.action_tx.clone().unwrap().send(Action::GotGeo(maybe_data, iomsg.clone(), true))?;  // return true, GeoData came from DB
        }
      },
      Action::GotGeo(x, iomsg, z) => {
        // Guard: if GeoData is from DB we return immediately, to not insert it again -> yes ofc insert it again.. how else to update u dingus?!
        //if z {return Ok(Option::None);} 
      
        let conn = self.dbconn.as_ref().unwrap();

        //let mut ip = ip::select_ip(conn, x.ip.as_str()).unwrap_or_default().unwrap_or_default(); // check if freshly acquired geodata has ip thats in db already, already done
        let ip_in_db: bool = z;

        //if ip == ip::IP::default() {
        //  ip_in_db = false;
        //}
        let mut ip = x.clone();

        let mut country = country::select_country(conn, x.country.as_str()).unwrap_or_default().unwrap_or_default();
        if country == country::Country::default() {
          let _ = country::insert_new_country(conn, x.country.as_str(), Some(x.countrycode.as_str()), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
        }
        else {
          country.warnings += 1;
          if !ip_in_db && x.is_banned {country.banned += 1;}
          let _ = country::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings), country.is_blocked).unwrap();
        }

        let mut region = region::select_region(conn, x.region.as_str()).unwrap_or_default().unwrap_or_default();
        if region == region::Region::default() {
          let _ = region::insert_new_region(conn, x.region.as_str(), x.country.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
        }
        else {
          region.warnings += 1;
          if !ip_in_db && x.is_banned {region.banned += 1;}
          let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), region.is_blocked).unwrap();
        }

        let mut city = city::select_city(conn, x.city.as_str()).unwrap_or_default().unwrap_or_default();
        if city == city::City::default() {
          let _ = city::insert_new_city(conn, x.city.as_str(), x.country.as_str(), x.region.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), false).unwrap();
        }
        else {
          city.warnings += 1;
          if !ip_in_db && x.is_banned {city.banned += 1;}
          let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(),city.region.as_str(), Some(city.banned), Some(city.warnings), city.is_blocked).unwrap();
        }

        let mut isp: isp::ISP = isp::select_isp(conn, x.isp.as_str()).unwrap_or_default().unwrap_or_default();
        if isp == isp::ISP::default() {
          let _ = isp::insert_new_ISP(conn, x.isp.as_str(), match x.is_banned {false => Some(0), true => Some(1)}, Some(1), x.country.as_str(), false).unwrap();
        }
        else {
          isp.warnings += 1;
          if !ip_in_db && x.is_banned {isp.banned += 1;}
          let _ = isp::insert_new_ISP(conn, isp.name.as_str(), Some(isp.banned), Some(isp.warnings), x.country.as_str(), isp.is_blocked).unwrap();
        }
        
        if !ip_in_db {
          let _ = ip::insert_new_IP(conn, 
            x.ip.as_str(), x.created_at.as_str(), 
            x.lon.as_str(), x.lat.as_str(), 
            x.isp.as_str(), x.city.as_str(), 
            Some(x.region.as_str()), x.country.as_str(),
            Some(x.countrycode.as_str()), x.banned_times, 
              x.is_banned, x.warnings).unwrap();
        }
        else {
          // ip is in db
          ip.warnings += 1;
          let _ = ip::insert_new_IP(conn,
            x.ip.as_str(), x.created_at.as_str(), 
            x.lon.as_str(), x.lat.as_str(), 
            x.isp.as_str(), x.city.as_str(), 
            Some(x.region.as_str()), x.country.as_str(),
            Some(x.countrycode.as_str()), x.banned_times, 
              x.is_banned, ip.warnings).unwrap();
        }

        let prod: IOProducer;

        let catmsg = match iomsg.clone() {
          IOMessage::SingleLine(x, p) => {
            prod = p;
            x
          },
          IOMessage::MultiLine(vx, p) => {
            prod = p;
            vx.join(" ")
          }
        };
        let is_jctl: bool = prod == IOProducer::Journal;
        let is_ban  = catmsg.contains("Ban");
        let tx = self.action_tx.clone().unwrap();
        tx.send(Action::PassGeo(ip.clone(), iomsg.clone(), z)).expect("PassGeo failed to send");
        let symb = if z {self.apptheme.symbol_db.clone()} else {self.apptheme.symbol_reqwest.clone()};
        let fetchmsg = format!(" {} Got location for IP {} ", symb, ip.ip);
        tx.send(Action::InternalLog(fetchmsg)).expect("Fetchlog message failed to send");

        if country.is_blocked || city.is_blocked || isp.is_blocked || region.is_blocked { 
          if !ip.is_banned && !is_ban {
            tx.send(Action::BanIP(x.clone())).expect("Block failed to send");
            let timestamp = chrono::offset::Local::now().to_rfc3339();
            let mut reasons: Vec<String> = vec![];
            if country.is_blocked {reasons.push(format!("Country: {}", country.name));}
            if region.is_blocked {reasons.push(format!("Region: {}", region.name));}
            if city.is_blocked {reasons.push(format!("City: {}", city.name));}
            if isp.is_blocked {reasons.push(format!("ISP: {}", isp.name));}
  
            let blockmsg = format!(" {} Blocked IP {} ", self.apptheme.symbol_block, ip.ip);
            tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");
            for reason in reasons {
              let blockmsg = format!(" {} Blocked {} ",self.apptheme.symbol_block , reason);
              tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");
            }
          } else {
            let blockmsg = format!(" {} IP already blocked {} ", self.apptheme.symbol_block, ip.ip);
            tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");            
          }       

        }

        let timestamp = chrono::offset::Local::now().to_rfc3339();
        match iomsg {
          IOMessage::SingleLine(msg, _) => {
            message::insert_new_message(conn, Option::None, &timestamp, &msg, &x.ip, &x.country, &x.region, &x.city, &x.isp, is_jctl, is_ban).unwrap();
          },
          IOMessage::MultiLine(vx, _) => {
            for msg in vx {
              message::insert_new_message(conn, Option::None, &timestamp, &msg, &x.ip, &x.country, &x.region, &x.city, &x.isp, is_jctl, is_ban).unwrap();
            }
          },
        }
        //self.stored_geo.push(x.clone()); 
      },
      Action::SubmitQuery(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        let ip = ip::select_ip(conn, x.as_str()).unwrap_or_default().unwrap_or_default();

        let tx = self.action_tx.clone().unwrap();

        let opmsgs = message::select_message_by_ip(conn, x.as_str()).unwrap();
        let mut actmsgs: Vec<message::Message> = vec![];

        for opmsg in opmsgs {
          let msg = opmsg.unwrap_or(message::Message::default());
          if msg != message::Message::default() {actmsgs.push(msg);}
        }

        if ip == ip::IP::default() {
          // send back query not found
          tx.send(Action::QueryNotFound(x)).expect("QueryNotFound failed to send!");
        } else {
          // spawn thread to send debounced messages
          tokio::spawn(async move{
            for msg in actmsgs {
              let prod = if msg.is_jctl {IOProducer::Journal} else {IOProducer::Log};
              tx.send(Action::PassGeo(ip.clone(), IOMessage::SingleLine(msg.text, prod), true)).expect("PassGeo failed to send on query!"); 
              tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;}
              // inefficient but else but require me to set up a duplicate receiver or refactor receive function
          });
        }        
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
        let fetchmsg = format!(" {} Blocked Country: {}", self.apptheme.symbol_block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block Country message failed to send");
      },
      Action::StatsUnblockCountry(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let country = country::select_country(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = country::insert_new_country(conn, country.name.as_str(), Some(country.code.as_str()),Some(country.banned), Some(country.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked Country: {}", self.apptheme.symbol_unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock Country message failed to send");
      },      
      Action::StatsBlockRegion(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let region = region::select_region(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), true).unwrap();
        let fetchmsg = format!(" {} Blocked Region: {}", self.apptheme.symbol_block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block Region message failed to send");
      },
      Action::StatsUnblockRegion(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let region = region::select_region(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = region::insert_new_region(conn, region.name.as_str(), region.country.as_str(),Some(region.banned), Some(region.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked Region: {}", self.apptheme.symbol_unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock Region message failed to send");
      }, 
      Action::StatsBlockCity(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let city = city::select_city(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(), city.region.as_str(),Some(city.banned), Some(city.warnings), true).unwrap();
        let fetchmsg = format!(" {} Blocked City: {}", self.apptheme.symbol_block, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block City message failed to send");
      },
      Action::StatsUnblockCity(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let city = city::select_city(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = city::insert_new_city(conn, city.name.as_str(), city.country.as_str(), city.region.as_str(),Some(city.banned), Some(city.warnings), false).unwrap();
        let fetchmsg = format!(" {} Unblocked City: {}", self.apptheme.symbol_unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unblock City message failed to send");
      },    
      Action::StatsBlockISP(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let isp = isp::select_isp(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as blocked
        let _ = isp::insert_new_ISP(conn, isp.name.as_str(),Some(isp.banned), Some(isp.warnings),isp.country.as_str(), true).unwrap();
        let fetchmsg = format!(" {} Blocked ISP: {}", self.apptheme.symbol_unblock, &x.name);
        tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Block ISP message failed to send");
      },
      Action::StatsUnblockISP(x) => {
        let conn = self.dbconn.as_ref().unwrap();
        // get item from db to see if it was updated, it will exist becasue we query from stats.
        let isp = isp::select_isp(conn, x.name.as_str()).unwrap_or_default().unwrap_or_default();
        // insert new as unblocked
        let _ = isp::insert_new_ISP(conn, isp.name.as_str(), Some(isp.banned), Some(isp.warnings),isp.country.as_str(), false).unwrap();
        let fetchmsg = format!(" {} Unblocked ISP: {}", self.apptheme.symbol_unblock, &x.name);
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
        let symb = self.apptheme.symbol_ban.clone();
        let _symb = self.apptheme.symbol_ban.clone();
        if !x.is_banned {

          let besure = self.f2b_check_banned(&x.ip);
          
          if !besure {        
            let tx = self.action_tx.clone().unwrap();
            
            //let cip = x.clone();
            tokio::spawn(async move {
              tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
              // check if is banned
              let output = std::process::Command::new("fail2ban-client")
                .arg("set")
                .arg("sshd")
                .arg("banip")
                .arg(&x.ip)
                // Tell the OS to record the command's output
                .stdout(std::process::Stdio::piped())
                // execute the command, wait for it to complete, then capture the output
                .output()
                // Blow up if the OS was unable to start the program
                .unwrap();
      
              // extract the raw bytes that we captured and interpret them as a string
              let stdout = String::from_utf8(output.stdout).unwrap();
              if stdout.contains("0") {
                tx.send(Action::Banned(true)).expect("Failed to Ban ...");
                let fetchmsg = format!(" {} Banned IP: {}", symb, &x.ip);
                tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Ban IP message failed to send");
              } else {
                let fetchmsg = format!(" {} Banned IP: {}", symb, &x.ip);
                tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Ban IP message failed to send");                 
                tx.send(Action::Banned(false)).expect("Failed to Ban ...");
              }
            });
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
          let symb = self.apptheme.symbol_unblock.clone();
          tokio::spawn(async move {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            // check if is banned
            let output = std::process::Command::new("fail2ban-client")
              .arg("set")
              .arg("sshd")
              .arg("unbanip")
              .arg(&x.ip)
              // Tell the OS to record the command's output
              .stdout(std::process::Stdio::piped())
              // execute the command, wait for it to complete, then capture the output
              .output()
              // Blow up if the OS was unable to start the program
              .unwrap();
    
            // extract the raw bytes that we captured and interpret them as a string
            let stdout = String::from_utf8(output.stdout).unwrap();
            if stdout.contains("0") {
              tx.send(Action::Unbanned(true)).expect("Failed to Unban !!!");
              let fetchmsg = format!(" {} Unbanned IP: {}", symb, &x.ip);
              tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unban IP message failed to send");
            } else {
              let fetchmsg = format!(" {} Unbanned IP: {}", symb, &x.ip);
              tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Unban IP message failed to send");
              tx.send(Action::Unbanned(false)).expect("Failed to Unban !!!"); // idfkgetit
            }
          });
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