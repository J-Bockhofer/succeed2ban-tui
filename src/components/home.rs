use std::{collections::HashMap, time::Duration, ops::Index};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent, ModifierKeyCode};
use futures::{TryFutureExt, FutureExt};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio_util::sync::CancellationToken;


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

#[derive(Clone, PartialEq)]
struct PointData {
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
struct IPListItem {
  pub IP: schema::IP,
  pub username: String,
  pub pointdata: PointData,
}
#[allow(non_snake_case)]
impl IPListItem {
  pub fn new(IP:schema::IP, username:String, pointdata: PointData)-> Self {
    IPListItem { IP, username, pointdata}
  }
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



  iplist: StatefulList<IPListItem>,

  infotext: String,
  elapsed_notify: usize,
  debug_me: String,


                                                      //f2b or journal // IP
  stored_styled_iostreamed: StatefulList<(StyledLine, String, String)>,
  //stored_styled_iostreamed: StatefulList<IOListItem>, Todo ?

  apptheme: themes::Theme,

  jctlrunning: bool,
  f2brunning: bool,

  last_username: String,

  startup_complete:bool,
  
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

    let mut visible_points: Vec<PointData> = vec![];

    for item in self.iplist.items.clone() {
      match self.drawmode {
        DrawMode::Sticky => {
          // push only item with selected ip
          if item.IP.ip == self.selected_ip {
            visible_points.push(item.pointdata);
          }
        },
        DrawMode::Decaying => {
          // decay only make visible when alive, refresh alive upon selection
          if item.pointdata.is_alive {
            visible_points.push(item.pointdata);
          }
        },
        DrawMode::All =>{
          // push every cloned item
          visible_points.push(item.pointdata);
        },
      }
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


            for pointdata in &visible_points {

              let x2 = pointdata.lon;
              let y2 =  pointdata.lat;
              let dir = (pointdata.dir_home_lon, pointdata.dir_home_lat);
            
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

            if self.iplist.items.len() == 0 {
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
              'J'|'j' => {self.last_mode = self.mode; Action::LogsPrevious}, // up // LogsSchedulePrevious Schedule locks me out with resetting modes
              'H'|'h' => {self.last_mode = self.mode; Action::LogsFirst}, // top
              'K'|'k' => {self.last_mode = self.mode; Action::LogsNext},  // down
              'L'|'l' => {self.last_mode = self.mode; Action::LogsLast}, // bottom
              'N'|'n' => {self.stored_styled_iostreamed.unselect(); Action::Blank}, // unselect
              // IP-ListNavigation
              // -- ArrowKeys


              _ => {//Action::Render
                Action::Blank}
            }
          },
          KeyCode::Down => {Action::IPsNext},
          KeyCode::Up => {Action::IPsPrevious},
          KeyCode::Right => {Action::EnterTakeAction},
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

        //////////////
      Mode::TakeAction => {
        match key.code {
          KeyCode::Tab => {
            Action::EnterNormal
          },
          KeyCode::Left => {
            self.available_actions.unselect();
            Action::EnterNormal 
          },
          KeyCode::Down => {Action::ActionsNext},
          KeyCode::Up => {Action::ActionsPrevious},
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
              'J'|'j' => {Action::LogsSchedulePrevious}, // up
              'H'|'h' => {Action::LogsScheduleLast}, // top
              'K'|'k' => {Action::LogsScheduleNext},  // down
              'L'|'l' => {Action::LogsScheduleFirst}, // bottom
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
      Action::EnterProcessing => { self.mode = Mode::Processing;}, // self.last_mode = self.mode;
      Action::ExitProcessing => {self.mode = self.last_mode;}, // TODO: Last mode, solved? No we have to look into the future to see if we want to change to same again and then forgo that

      // List Actions
      // -- LOG LIST -- iostreamed
      Action::LogsScheduleNext => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsNext);}, // deprec
      Action::LogsNext => {if self.stored_styled_iostreamed.items.len() > 0 {self.stored_styled_iostreamed.next();}},
      Action::LogsSchedulePrevious => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsPrevious);}, // {list_actions::schedule_previous_loglist(self.command_tx.clone().unwrap());},
      Action::LogsPrevious => {if self.stored_styled_iostreamed.items.len() > 0 {self.stored_styled_iostreamed.previous();}},
      Action::LogsScheduleFirst => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsFirst);}, // deprec
      Action::LogsFirst => {  self.stored_styled_iostreamed.state.select(Some(0)) },
      Action::LogsScheduleLast => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsLast);}, // deprec
      Action::LogsLast => {  let idx = Some(self.stored_styled_iostreamed.items.len() - 1);
        self.stored_styled_iostreamed.state.select(idx); },
      // -- IP LIST -- iplist
      Action::IPsScheduleNext => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::IPsNext);}, // deprec
      Action::IPsNext => {            
        if self.iplist.items.len() > 0 {
          self.iplist.next();
          let sel_idx = self.iplist.state.selected().unwrap();
          self.iplist.items[sel_idx].pointdata.refresh();
          self.selected_ip =  self.iplist.items[sel_idx].IP.ip.clone();
        }
      },
      Action::IPsSchedulePrevious => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::IPsPrevious);}, // deprec
      Action::IPsPrevious => {            
        if self.iplist.items.len() > 0 {
          self.iplist.previous();
          let sel_idx = self.iplist.state.selected().unwrap();
          self.iplist.items[sel_idx].pointdata.refresh();
          self.selected_ip =  self.iplist.items[sel_idx].IP.ip.clone();
        }
      },
      // ACTION LIST self.available_action
      Action::ActionsNext => {self.available_actions.next();},
      Action::ActionsPrevious => {self.available_actions.previous();},
      Action::ActionsExecute => {
        let action_idx = self.available_actions.state.selected().unwrap();
        match self.available_actions.items[action_idx].0 {
          "Ban" => {self.command_tx.clone().unwrap().send(Action::Ban)?;},
          "monitor-fail2ban" => {
            // check if is active
            if self.f2brunning {
              self.available_actions.items[action_idx].1 = String::from("inactive");
              self.command_tx.clone().unwrap().send(Action::StopF2BWatcher)?;
            } else {
              self.available_actions.items[action_idx].1 = String::from("active");
              self.f2brunning = true;
              self.command_tx.clone().unwrap().send(Action::StartF2BWatcher)?;                          
            }
          },
          "monitor-journalctl" => {
            // check if is active
            if self.jctlrunning{
              // switch to inactive
              self.available_actions.items[action_idx].1 = String::from("inactive");
              self.command_tx.clone().unwrap().send(Action::StopJCtlWatcher)?;           
            }
            else{
              // switch to active
              self.available_actions.items[action_idx].1 = String::from("active");
              self.jctlrunning = true;
              self.command_tx.clone().unwrap().send(Action::StartJCtlWatcher)?;            
            }},
            _ => {},
          }},


      Action::StoppedJCtlWatcher => {self.jctlrunning = false;},
      Action::IONotify(x) => {self.elapsed_notify += 1;},
      Action::GotGeo(x,y) => {

        self.style_incoming_message(y.clone());

        let cip = x.ip.clone();

        let cipvec = self.iplist.items.clone();

        if !cipvec.iter().any(|i| i.IP.ip==cip) {
          // if cip isnt in vector yet
          let lat = x.lat.clone().parse::<f64>().unwrap();
          let lon = x.lon.clone().parse::<f64>().unwrap();
          let dir_lat = self.home_lat - lat;
          let dir_lon = self.home_lon - lon;

          self.last_lat = lat.clone();
          self.last_lon = lon.clone();
  
          self.last_direction = (dir_lon, dir_lat);

          let pointdata = PointData::new(cip.clone(), lon, lat, dir_lon, dir_lat);

          let iplistitem = IPListItem::new(x.clone(), self.last_username.clone(), pointdata);

          //self.iplist.items.push((cip.clone(), x.clone(), self.last_username.clone()));
          self.iplist.items.push(iplistitem);
          self.iplist.trim_to_length(10); // change to const

          if self.iplist.items.len() > 1 {
            self.iplist.state.select(Option::Some(self.iplist.items.len()-1));
          }
          else {
            self.iplist.state.select(Option::Some(0));
          }
          self.selected_ip = cip;
          


        } 
        else {
          // ip is already in vector, need to select it again
          for i in 0..self.iplist.items.len() {
            let item = &self.iplist.items[i];
            if item.IP.ip == cip {
              self.iplist.state.select(Some(i));
              self.iplist.items[i].pointdata.refresh();
              self.selected_ip = cip;
              break;
            }
          }
        }    
      },
      Action::Ban => {
        let sel_ip = self.iplist.state.selected();
        let banip: String;
        if sel_ip.is_some() {
          banip = self.iplist.items[sel_ip.unwrap()].IP.ip.clone();
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

      for item in &mut self.iplist.items {
        item.pointdata.decay_point(self.apptheme.decay_time);
      }
      
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
          let mut lines = vec![Line::from(format!("{}   {}", i.IP.ip, i.username))]; // let mut lines = vec![Line::from(i.0)];
          let mut symb = "X";
          if i.IP.is_banned  {
            symb = "✓";
          }
          lines.push(
            format!("{} - {}, {}", symb, i.IP.city, i.IP.country)
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
  
        
        let infoblock = Paragraph::new(format!("Received I/O: {} -{}-numLinesLast- dbg-msg:   {}",self.infotext.clone(), self.elapsed_notify.to_string(), self.debug_me))
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

