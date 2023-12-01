use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, ModifierKeyCode};
use futures::{TryFutureExt, FutureExt};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;


use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  geofetcher, gen_structs::StatefulList,
  themes, animations, migrations::schema,
  action_handlers::list_actions,
};

use tui_input::{backend::crossterm::EventHandler, Input};
use log::error;

use regex::Regex;

#[derive(Default, Clone)]
struct StyledLine {
  words: Vec<(String, Style)>,
}

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
  to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}



#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  TakeAction,
  Processing,
}


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DrawMode {
  #[default]
  Sticky,
  Decaying,
  All,
}


#[derive(Default)]
pub struct Home<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  items: StatefulList<(&'a str, usize)>,
  available_actions: StatefulList<(&'a str, String)>,
  //iostreamed: StatefulList<(String, usize)>, // do i need a tuple here? // CHANGED
  //iostreamed: Vec<(String, usize)>,
  iplist: StatefulList<(String, schema::IP, String)>,
  pub last_events: Vec<KeyEvent>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub input: Input,
  pub mode: Mode,
  pub drawmode: DrawMode,
  elapsed_rticks: usize,
  elapsed_frames: f64,

  selected_ip: String,
  last_lat: f64,
  last_lon: f64,

  home_lat: f64,
  home_lon: f64,

  last_direction: (f64, f64), //tuple vector2D that points towards home; 0/x = lon, 1/y = lat
  // home - last 
  point_dir_vec: Vec<((f64, f64), (f64, f64), Option<tokio::time::Instant>)>,


  infotext: String,
  elapsed_notify: usize,
  debug_me: String,

  //styledio: Vec<ListItem<'a>>,
  //styledio: Vec<StyledLine>,

  //stored_styled_lines: Vec<StyledLine>,
                                                      //f2b or journal // IP
  stored_styled_iostreamed: StatefulList<(StyledLine, String, String)>,

  apptheme: themes::Theme,

  jctlrunning: bool,
  f2brunning: bool,

  last_username: String,

  startup_complete:bool,

  time_last: Option<tokio::time::Instant>,
  
  last_mode: Mode,

}

impl<'a> Home<'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  pub fn set_items(mut self) -> Self {
    self.iplist = StatefulList::with_items(vec![
    ]);  
    self.items = StatefulList::with_items(vec![
      ("Item0", 1),
      ("Item1", 2),
      ("Item2", 1),
      ("Item3", 1),
      ("Item4", 1),
      ("Item5", 2),
      ("Item6", 1),
      ("Item7", 1),
    ]);   
    self.available_actions = StatefulList::with_items(vec![
      ("Ban", String::from("some ip")),
      ("Unban", String::from("some ip")),
      ("monitor-journalctl", String::from("inactive")),
      ("monitor-fail2ban", String::from("inactive")),
    ]);
    self.last_lat = 53.0416;
    self.last_lon = 8.9433;
    self.point_dir_vec = vec![];


    self.home_lat = 53.0416;
    self.home_lon = 8.9433;
    self.apptheme = themes::Theme::default();
    self.jctlrunning = false;
    self.f2brunning = false;
    self.startup_complete = false;
    self.drawmode = DrawMode::Decaying;
  
    self
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  fn map_canvas(&self, area: &Rect) -> impl Widget + '_ {

    let w = f64::from(area.width.clone());
    let h = f64::from(area.height.clone());

    let mut num_iter: usize;
    let mut decayedvec: Vec<bool> = vec![];
    for _ in 0..self.point_dir_vec.len() {
      decayedvec.push(false);
    }

    match self.drawmode {
      DrawMode::Sticky => {num_iter = 1; if self.point_dir_vec.len() == 0 {num_iter = 0;}}, // single line stays forever
      DrawMode::Decaying => {
        
        for i in 0..self.point_dir_vec.len() {
          let time_last = self.point_dir_vec[i].2.unwrap_or_else(||{tokio::time::Instant::now()});
          if time_last.elapsed() > self.apptheme.decay_time {
            decayedvec[i] = true;
          } 
        }
        num_iter = self.point_dir_vec.len();

      }, // single line decays
      DrawMode::All => {num_iter = self.point_dir_vec.len();}, // all lines sticky
    }

    canvas::Canvas::default()
        .background_color(self.apptheme.colors.default_background)
        .block(Block::default().borders(Borders::ALL).title("World").bg(self.apptheme.colors.default_background))
        .marker(Marker::Braille)
        .paint(move |ctx| {
/*             ctx.draw(&canvas::Rectangle {
              x:  w / 2.,
              y: h / 2.,
              width: w,
              height: h,
              color:self.apptheme.colors.default_background,
            }); */

            ctx.draw(&canvas::Map {
                color: self.apptheme.colors.default_map_color,
                resolution: canvas::MapResolution::High,
            });

            for i in 0..num_iter {
              let idx = self.point_dir_vec.len() - i - 1;

              if decayedvec[i] {
                continue;
              }
              
              let x2 = self.point_dir_vec[idx].0.0;
              let y2 =  self.point_dir_vec[idx].0.1;
              let dir = self.point_dir_vec[idx].1;

            
              // draw line to home
              ctx.draw(&canvas::Line {
                x1: self.home_lon,
                y1: self.home_lat,
                x2: x2,
                y2: y2,
                color:self.apptheme.colors.accent_dblue,
              }); 

              ctx.draw(&canvas::Line {
                x1: x2 + dir.0 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
                y1: y2 + dir.1 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
                x2: x2,
                y2: y2,
                color: self.apptheme.colors.accent_blue,
              });
            
              ctx.draw(&canvas::Circle {
                x: x2, // lon
                y: y2, // lat
                radius: self.elapsed_frames,
                color: self.apptheme.colors.accent_orange,
              });

            }

            if num_iter == 0 {
              ctx.draw(&canvas::Circle {
                x: self.last_lon, // lon
                y: self.last_lat, // lat
                radius: self.elapsed_frames,
                color: self.apptheme.colors.accent_orange,
              });

            }
            


            //ctx.print(self.last_lon, self.last_lat, "X".red());
            ctx.print(self.home_lon, self.home_lat, Line::from(Span::styled("H", Style::default().fg(self.apptheme.colors.accent_orange))));
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
  }

  


  pub fn style_incoming_message(&mut self, msg: String) {
    let mut dbg: String;

    let mut last_io = String::from("Journal");
    self.last_username = String::from("");

    if msg.contains("++++") {
      // message is from Fail2Ban
      last_io = String::from("Fail2Ban");
    }
    let collected: Vec<&str> = msg.split("++++").collect(); // new line delimiter in received lines, if more than one got added simultaneously
    self.debug_me = collected.clone().len().to_string();
    //self.debug_me = collected.clone().len().to_string();

    for tmp_line in collected {
      if tmp_line.is_empty() {
        continue;
      }
      let mut thisline: StyledLine = StyledLine::default();
      // do word_map matching first then regex match splitting
      // look for ip quickly to send it out to the list
      let results: Vec<&str> = self.apptheme.ipregex
      .captures_iter(tmp_line)
      .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
      .collect();
      let mut cip: &str = "";
      if !results.is_empty() {
        cip = results[0];
      }


      let words: Vec<&str> = tmp_line.split(" ").collect();
      let mut held_unstyled_words: Vec<&str> = vec![];
      let mut last_word: &str= "";

      for word in words.clone(){
        let mut word_style = self.apptheme.word_style_map.get_style_or_default(word.to_string()); // Detector for constant word
        if word_style == Style::default() {
          // try regex styling on word
          word_style = self.apptheme.regex_style_map.get_style_or_default(word.to_string()); // Detector for regex
        } 
        if last_word == "user" {
          word_style = self.apptheme.username_style;
          self.last_username = word.to_string();
        }
        

        if word_style == Style::default() {
          // If no detector has returned any styling
          held_unstyled_words.push(word);
        }
        else {
          // word is styled
          // if there are any held words push them with default style and reset held words
          if held_unstyled_words.len() > 0 {

            thisline.words.push((format!(" {}", held_unstyled_words.join(" ")), self.apptheme.default_text_style));
            held_unstyled_words = vec![];
          }
          // push styled word with space in front - TODO word is in first position
          thisline.words.push((format!(" {}", word.to_string()), word_style));

        }
        last_word = word;

        // terminate
        if &word == words.last().unwrap() {
          thisline.words.push((format!(" {}",held_unstyled_words.join(" ")), self.apptheme.default_text_style));
        }
        

      }

      
      
      //self.stored_styled_lines.push(thisline);
      self.stored_styled_iostreamed.items.push((thisline, last_io.clone(), cip.to_string()));
      self.stored_styled_iostreamed.trim_to_length(100);

    }// end per line

  }



}

impl Component for Home<'_> {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.command_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Processing => return Ok(None),
      Mode::Normal => {
        match key.code {
          KeyCode::Esc => Action::Quit,
          KeyCode::Char(keychar) => {
            match keychar {
              // DrawMode switching
              'A'|'a' => {self.drawmode = DrawMode::All; Action::Blank},
              'S'|'s' => {self.drawmode = DrawMode::Sticky; Action::Blank},
              'D'|'d' => {self.drawmode = DrawMode::Decaying; Action::Blank},
              // IO-ListNavigation
              'J'|'j' => {self.last_mode = self.mode; Action::LogsSchedulePrevious}, // up
              'H'|'h' => {self.last_mode = self.mode; Action::LogsScheduleFirst}, // top
              'K'|'k' => {self.last_mode = self.mode; Action::LogsScheduleNext},  // down
              'L'|'l' => {self.last_mode = self.mode; Action::LogsScheduleLast}, // bottom
              'N'|'n' => {self.stored_styled_iostreamed.unselect(); Action::Blank}, // unselect
              // IP-ListNavigation
              // -- ArrowKeys


              _ => {//Action::Render
                Action::Blank}
            }
          },
          KeyCode::Down => {
            //println!("Arrow Down");
            if self.iplist.items.len() > 0 {
              self.iplist.next();
            
              let sel_idx = self.iplist.state.selected().unwrap();
              self.selected_ip = self.iplist.items[sel_idx].0.clone();
              let lat = self.iplist.items[sel_idx].1.lat.clone().parse::<f64>().unwrap();
              let lon = self.iplist.items[sel_idx].1.lon.clone().parse::<f64>().unwrap();
  
              self.last_lat = lat;
              self.last_lon = lon;
  
              let last_direction = (self.home_lon - self.last_lon, self.home_lat - self.last_lat);

              self.last_direction = last_direction;
              self.point_dir_vec.push(((lon, lat), last_direction, Some(tokio::time::Instant::now())));
              if self.point_dir_vec.len() > 10 {
                while self.point_dir_vec.len() > 10 {
                  self.point_dir_vec.remove(0);
                }
              }                
            }


            //Action::Render
            Action::Blank
          },
          KeyCode::Up => {
            if self.iplist.items.len() > 0 {
              self.iplist.previous();
            
              let sel_idx = self.iplist.state.selected().unwrap();
              self.selected_ip = self.iplist.items[sel_idx].0.clone();
              let lat = self.iplist.items[sel_idx].1.lat.clone().parse::<f64>().unwrap();
              let lon = self.iplist.items[sel_idx].1.lon.clone().parse::<f64>().unwrap();

              self.last_lat = lat;
              self.last_lon = lon;

              self.last_direction = (self.home_lon - self.last_lon, self.home_lat - self.last_lat);

              self.point_dir_vec.push(((self.last_lon,self.last_lat),self.last_direction, Some(tokio::time::Instant::now())));
              if self.point_dir_vec.len() > 10 {
                while self.point_dir_vec.len() > 10 {
                  self.point_dir_vec.remove(0);
                }
              }              

            }
            //Action::Render
            Action::Blank
          },
          KeyCode::Right => {
            Action::EnterTakeAction
          },
          KeyCode::Left => {
            self.iplist.unselect();
            self.selected_ip = "".to_string();
            //Action::Render
            Action::Blank
          },
          KeyCode::Tab => {
            Action::EnterTakeAction
            
          },
          KeyCode::Enter => {
            Action::EnterTakeAction
          },
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            //Action::Render
            Action::Blank
          },
        }}
      Mode::TakeAction => {
        match key.code {
          KeyCode::Tab => {
            Action::EnterNormal
          },
          KeyCode::Left => {
            self.available_actions.unselect();
            Action::EnterNormal 
          },
          KeyCode::Down => {
            //println!("Arrow Down");

            self.available_actions.next(); 

            //Action::Render
            Action::Blank
          },
          KeyCode::Up => {

            self.available_actions.previous();

            //Action::Render
            Action::Blank
          },
          KeyCode::Right => {
            let action_idx = self.available_actions.state.selected().unwrap();
            match self.available_actions.items[action_idx].0 {
              "Ban" => {Action::Ban},
              "monitor-fail2ban" => {
                // check if is active
                if self.f2brunning {
                  self.available_actions.items[action_idx].1 = String::from("inactive");
                  Action::StopF2BWatcher
                } else {
                  self.available_actions.items[action_idx].1 = String::from("active");
                  self.f2brunning = true;
                  Action::StartF2BWatcher                 
                }

              },
              "monitor-journalctl" => {
                // check if is active
                if self.jctlrunning{
                  // switch to inactive
                  self.available_actions.items[action_idx].1 = String::from("inactive");
                  Action::StopJCtlWatcher
                }
                else{
                  // switch to active
                  self.available_actions.items[action_idx].1 = String::from("active");
                  self.jctlrunning = true;
                  Action::StartJCtlWatcher  
                }
              },
              _ => {//Action::Render
                Action::Blank},
            }
          }, 
          KeyCode::Enter => {
            let action_idx = self.available_actions.state.selected().unwrap();
            match self.available_actions.items[action_idx].0 {
              "Ban" => {Action::Ban},
              "monitor-fail2ban" => {
                // check if is active
                if self.available_actions.items[action_idx].1.as_str() == "active"{
                  // switch to inactive
                  self.available_actions.items[action_idx].1 = String::from("inactive");
                  Action::StopF2BWatcher
                }
                else{
                  // switch to active
                  self.available_actions.items[action_idx].1 = String::from("active");
                  Action::StartF2BWatcher
                }
              },
              "monitor-journalctl" => {
                // check if is active
                if self.available_actions.items[action_idx].1.as_str() == "active"{
                  // switch to inactive
                  self.available_actions.items[action_idx].1 = String::from("inactive");
                  Action::StopJCtlWatcher
                }
                else{
                  // switch to active
                  self.available_actions.items[action_idx].1 = String::from("active");
                  Action::StartJCtlWatcher
                }

              },
              _ => {//Action::Render
                Action::Blank},
            }
          },
          KeyCode::Char(keychar) => {
            match keychar {
              // DrawMode switching
              'A'|'a' => {self.drawmode = DrawMode::All; Action::Blank},
              'S'|'s' => {self.drawmode = DrawMode::Sticky; Action::Blank},
              'D'|'d' => {self.drawmode = DrawMode::Decaying; Action::Blank},
              // IO-ListNavigation
              'J'|'j' => {self.last_mode = self.mode; Action::LogsSchedulePrevious}, // up
              'H'|'h' => {self.last_mode = self.mode; Action::LogsScheduleLast}, // top
              'K'|'k' => {self.last_mode = self.mode; Action::LogsScheduleNext},  // down
              'L'|'l' => {self.last_mode = self.mode; Action::LogsScheduleFirst}, // bottom
              'N'|'n' => {self.stored_styled_iostreamed.unselect(); Action::Blank}, // unselect


              _ => {//Action::Render
                Action::Blank}
            }
          },       
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            //Action::Render
            Action::Blank
          },
        }
      },
    };
    
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {

    match action {
      Action::Tick => {
      },
      Action::StartupDone => {self.startup_complete = true; self.command_tx.clone().unwrap().send(Action::Refresh)?;}
      Action::EnterNormal => {self.mode = Mode::Normal; self.last_mode = self.mode;},
      Action::EnterTakeAction => {self.mode = Mode::TakeAction; self.last_mode = self.mode;},
      Action::EnterProcessing => {self.mode = Mode::Processing;},
      Action::ExitProcessing => {self.mode = self.last_mode;}, // TODO: Last mode, solved?

      // List Actions
      // -- LOG LIST -- iostreamed
      Action::LogsScheduleNext => {list_actions::schedule_next_loglist(self.command_tx.clone().unwrap());},
      Action::LogsNext => {self.stored_styled_iostreamed.next();},
      Action::LogsSchedulePrevious => {list_actions::schedule_previous_loglist(self.command_tx.clone().unwrap());},
      Action::LogsPrevious => {self.stored_styled_iostreamed.previous();},
      Action::LogsScheduleFirst => {list_actions::schedule_first_loglist(self.command_tx.clone().unwrap());},
      Action::LogsFirst => {  self.stored_styled_iostreamed.state.select(Some(0)) },
      Action::LogsScheduleLast => {list_actions::schedule_last_loglist(self.command_tx.clone().unwrap());},
      Action::LogsLast => {  let idx = Some(self.stored_styled_iostreamed.items.len() - 1);
        self.stored_styled_iostreamed.state.select(idx); },
      // -- IP LIST -- iplist



      Action::StoppedJCtlWatcher => {self.jctlrunning = false;},
      Action::IONotify(x) => {

        self.elapsed_notify += 1;
       

        //return Ok(Some(Action::GetGeo));
        //self.command_tx.clone().unwrap().send(Action::GetGeo);
        
      },
      Action::GotGeo(x,y) => {

        self.style_incoming_message(y.clone());

        let cip = x.ip.clone();
        //let city = x.city;
        //let country = x.country;
        //let country_code = x.country_code;
        //let isp = x.isp;
        //let region_name = x.region_name;
        //self.time_last = Some(tokio::time::Instant::now());
        let geolat = x.lat.clone();
        let geolon = x.lon.clone();

        self.last_lat = geolat.parse::<f64>().unwrap();
        self.last_lon = geolon.parse::<f64>().unwrap();

        self.last_direction = (self.home_lon - self.last_lon, self.home_lat - self.last_lat);

        // : Vec<((f64, f64), (f64, f64))>,
        self.point_dir_vec.push(((self.last_lon,self.last_lat),self.last_direction, Some(tokio::time::Instant::now())));

        let cipvec = self.iplist.items.clone();

        if !cipvec.iter().any(|i| i.0==cip) {
          // if cip isnt in vector yet

          self.iplist.items.push((cip.clone(), x.clone(), self.last_username.clone()));
          self.iplist.trim_to_length(10); // change to const
          if self.point_dir_vec.len() > 10 {
            while self.point_dir_vec.len() > 10 {
              self.point_dir_vec.remove(0);
            }
          }

          if self.iplist.items.len() > 1 {
            self.iplist.state.select(Option::Some(self.iplist.items.len()-1));
          }
          else {
            self.iplist.state.select(Option::Some(0));
          }
          self.selected_ip = cip;
          


        }        
      },
      Action::Ban => {
        let sel_ip = self.iplist.state.selected();
        let banip: String;
        if sel_ip.is_some() {
          banip = self.iplist.items[sel_ip.unwrap()].0.clone();
          self.command_tx.clone().unwrap().send(Action::BanIP(banip))?;

        } else {
          todo!()
        }
      },
      Action::Banned(x) => {
        if x {self.infotext = String::from("BANNED");}
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    
    if self.startup_complete {

      self.elapsed_rticks += 1;
      self.elapsed_frames += 1.;
      const ANIMTICKS: usize = 4;
      let animsymbols = vec!["|","/","―","\\"];
  
      if self.elapsed_rticks >= ANIMTICKS {
        self.elapsed_rticks = 0;
        
        //self.infotext = String::from("");
      }
  
      if self.elapsed_frames >= 8. {
        self.elapsed_frames = 0.;
      }
  
      let layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
        .split(f.size());
  
  
      let left_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)])
        .split(layout[0]);
  
      let right_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
        .split(layout[1]);
  
      let av_actions: Vec<ListItem> = self
      .available_actions
      .items
      .iter()
      .map(|i| {
          let mut lines = vec![Line::from(i.0)];
          if i.0 == "Ban" || i.0 == "Unban"
          {
            lines.push(
              format!("X - {}", self.selected_ip)
                  .italic()
                  .into(),
            );          
          }
          else {
            let mut symb = "X";
            if i.1 == String::from("active") {
              symb = "✓";
            }
            lines.push(
                format!("{} - {}", symb, i.1)
                .italic()
                .into(),
            );            
          }
  
  
          ListItem::new(lines).style(Style::default().fg(Color::White))
      })
      .collect();
  
      // Create a List from all list items and highlight the currently selected one
      let actionlist = List::new(av_actions)
          .bg(self.apptheme.colors.lblack)
          .block(Block::default()
          .borders(Borders::ALL)
          .border_style( 
            match self.mode {
              Mode::TakeAction => {self.apptheme.active_border_style},
              _ => {  let mut style = self.apptheme.border_style;
                if self.last_mode == Mode::TakeAction {style = self.apptheme.active_border_style}
                style},
            })
          .title("Actions"))
          .highlight_style(self.apptheme.highlight_item_style)
          .highlight_symbol(">> ");
  
  
      let ips: Vec<ListItem> = self
      .iplist      // .items
      .items
      .iter()
      .map(|i| {
          let mut lines = vec![Line::from(format!("{}   {}", i.0, i.2))]; // let mut lines = vec![Line::from(i.0)];
          let mut symb = "X";
          if i.1.is_banned  {
            symb = "✓";
          }
          lines.push(
            format!("{} - {}, {}", symb, i.1.city, i.1.country)
                .italic()
                .into(),
          );
          ListItem::new(lines).style(Style::default().fg(Color::White))
      })
      .collect();
  
      // Create a List from all list items and highlight the currently selected one
      let iplist = List::new(ips)
          .bg(self.apptheme.colors.lblack)
          .block(Block::default()
          .borders(Borders::ALL)
          .border_style( 
            match self.mode {
              Mode::Normal => {self.apptheme.active_border_style},
              _ => {  let mut style = self.apptheme.border_style;
                if self.last_mode == Mode::Normal {style = self.apptheme.active_border_style}
                style},
            })
          .title("Last IPs"))
          .highlight_style(self.apptheme.highlight_item_style)
          .highlight_symbol(">> ");
  
        let term_w = right_layout[1].width as usize;
  
        //self.styledio == old
        let iolines: Vec<ListItem> = self
          .stored_styled_iostreamed
          .items 
          .iter()
          .map(|i| {
  
            let mut line: Line = Line::default();
            for word in i.0.words.clone() {
              let cspan = Span::styled(word.0, word.1); 
              line.spans.push(cspan);
            }

            let mut bg_style: Style;
            if i.1 == "Journal" {
              bg_style = self.apptheme.journal_bg;
            } else {
              bg_style = self.apptheme.fail2ban_bg;
            }

            if i.2 == self.selected_ip {
              bg_style = self.apptheme.selected_ip_bg;
            }

            let line_w = line.width();
            if line_w < term_w {
              // fill line with whitespaces
              let dif = term_w - line_w;
              let cspan = Span::styled(str::repeat(" ", dif), self.apptheme.default_text_style); 
              line.spans.push(cspan);
  
            }
            line.patch_style(bg_style);
            ListItem::new(line)
          })
          .collect();
  
        
  
  /*       let iolines: Vec<ListItem> = self
          .iostreamed
          .items // change stateful list to simple vector CHANGED
          .iter()
          .map(|i| {
            let stringo = format!("\033[92m{}\x1b[0m",i.0.as_str());
            ListItem::new(Line::from(i.0.as_str())).style(Style::default().fg(Color::White))
          })
          .collect(); */
        let mut ioactive: u8 = 0;
        if self.available_actions.items[2].1 == "active" || self.available_actions.items[3].1 == "active" {
          if self.available_actions.items[2].1 == "active" && self.available_actions.items[3].1 == "active" {
            ioactive = 2;
          } else{
            ioactive = 1;
          }
        }
  
  
        let iolist_title = Line::from(vec![
          Span::styled(" I/O Stream [ ", Style::default().fg(Color::White)),
          Span::styled(animsymbols[self.elapsed_rticks],
            match ioactive { 0 => {Style::default().fg(self.apptheme.colors.accent_wred)}, 1 => {Style::default().fg(self.apptheme.colors.accent_lpink)}, 2 => {Style::default().fg(self.apptheme.colors.accent_blue)} _ => {Style::default().fg(self.apptheme.colors.accent_wred)}}),
          Span::styled(" ] ", Style::default().fg(Color::White)),
        ]);
  
        // list
        // right_layout[1]
            
  
        // Create a List from all list items and highlight the currently selected one
        let iolist = List::new( iolines) //self.styledio.clone()
            .block(Block::default()
              .bg(self.apptheme.colors.lblack)
              .borders(Borders::ALL)
              .border_style( 
                match self.mode {
                  Mode::Normal => {Style::new().white()},
                  _ => Style::new().white(),
                })
              .title(iolist_title)
            )
            //.highlight_style(self.apptheme.highlight_item_style)
            .highlight_symbol(">> ");
  
        
        let infoblock = Paragraph::new(format!("{}-numIn-{}-numLinesLast-{}",self.infotext.clone(), self.elapsed_notify.to_string(), self.debug_me))
          .set_style(Style::default())
          .block(Block::default()
          .bg(self.apptheme.colors.lblack)
          .borders(Borders::ALL)
          .title("Info"));
  
      // Draw Map to right_upper = 0
      f.render_widget(self.map_canvas(&right_layout[0]), right_layout[0]);
  
      // Draw Read file to right_lower = 1
      f.render_stateful_widget(iolist, right_layout[1], &mut self.stored_styled_iostreamed.state); // CHANGED 
      // f.render_widget(iolist, right_layout[1]);
      
      f.render_widget(infoblock, left_layout[0]);
  
      f.render_stateful_widget(iplist, left_layout[1], &mut self.iplist.state);
  
      f.render_stateful_widget(actionlist, left_layout[2], &mut self.available_actions.state);
    } else {
      f.render_widget(Paragraph::new("hello world"), area);
    }



    Ok(())
  }

  


}

