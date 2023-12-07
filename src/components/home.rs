pub mod ui;
pub mod utils;
use utils::{centered_rect, map_range};

pub mod structs;
use structs::{StyledLine, PointData, IPListItem};

pub mod actions;
use actions::{style_incoming_message, parse_passed_geo};

pub mod enums;
use enums::*;

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
  migrations::schema::ip::IP,
  action_handlers::list_actions,
  animations::Animation, components::home::ui::create_internal_logs,
};

use tui_input::{backend::crossterm::EventHandler, Input};
use log::error;

use regex::Regex;



#[derive(Default)]
pub struct Home<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  //items: StatefulList<(&'a str, usize)>,
  available_actions: StatefulList<(&'a str, String)>,
  //iostreamed: StatefulList<(String, usize)>, // do i need a tuple here? // CHANGED
  //iostreamed: Vec<(String, usize)>,
  
  pub last_events: Vec<KeyEvent>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub input: Input,
  pub mode: Mode,
  pub drawmode: DrawMode,
  pub displaymode: DisplayMode,
  pub iomode: IOMode,

  elapsed_rticks: usize,
  elapsed_frames: f64,

  selected_ip: String,
  last_lat: f64,
  last_lon: f64,

  home_lat: f64,
  home_lon: f64,

  last_direction: (f64, f64), //tuple vector2D that points towards home; 0/x = lon, 1/y = lat
  // home - last 

  internal_logs: StatefulList<String>,


  iplist: StatefulList<IPListItem>,
  iplist_capacity: usize,

  infotext: String,
  elapsed_notify: usize,
  debug_me: String,


                                                      //f2b or journal // IP
  stored_styled_iostreamed: StatefulList<(StyledLine, String, String)>,
  //stored_styled_iostreamed: StatefulList<IOListItem>, Todo ?
  iostreamed_capacity: usize,
  iostreamed_capacity_input: String,

  apptheme: themes::Theme,

  jctlrunning: bool,
  f2brunning: bool,

  last_username: String,

  startup_complete:bool,
  showing_stats: bool,
  
  last_mode: Mode,

  querystring: String,
  queryerror: String,
  anim_querycursor: Animation<&'a str>,

  ipstring: String,
  iperror: String,

  available_themes: themes::Themes,

}

impl<'a> Home<'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  pub fn set_items(mut self) -> Self {
    self.iplist = StatefulList::with_items(vec![
    ]);
    self.internal_logs = StatefulList::with_items(vec![]);
    self.iplist_capacity = 10;
    self.available_actions = StatefulList::with_items(vec![
      ("Ban", String::from("some ip")),
      ("Unban", String::from("some ip")),
      ("monitor-journalctl", String::from("inactive")),
      ("monitor-fail2ban", String::from("inactive")),
      ("Stats", String::from("E | e")),
      ("Query", String::from("Q | q")),
      ("Help", String::from("W | w")),
      ("Exit", String::from("Esc | Ctrl+C")),
    ]);
    self.last_lat = 53.0416;
    self.last_lon = 8.9433;

    self.home_lat = 53.0416;
    self.home_lon = 8.9433;
    self.apptheme = themes::Theme::default();
    self.jctlrunning = false;
    self.f2brunning = false;
    self.startup_complete = false;
    self.showing_stats = false;
    self.drawmode = DrawMode::Decaying;
    self.displaymode = DisplayMode::Normal;
    self.iomode = IOMode::Follow;

    self.anim_querycursor = Animation::with_items(vec![""," "]);
    self.queryerror = String::from("Enter IP");

    self.iostreamed_capacity = 100;
    self.available_themes = themes::Themes::default();

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
            // draw map
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
                x2,
                y2,
                color:self.apptheme.colors.accent_dblue,
              }); 
              // draw animated line
              ctx.draw(&canvas::Line {
                x1: x2 + dir.0 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
                y1: y2 + dir.1 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
                x2,
                y2,
                color: self.apptheme.colors.accent_blue,
              });
              // draw animated circle
              ctx.draw(&canvas::Circle {
                x: x2, // lon
                y: y2, // lat
                radius: self.elapsed_frames,
                color: self.apptheme.colors.accent_orange,
              });

            }
            // if nothing is in ip list ie. on startup show a circle around the home coordinates
            if self.iplist.items.len() == 0 {
              ctx.draw(&canvas::Circle {
                x: self.home_lon, // lon
                y: self.home_lat, // lat
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

  fn add_to_querystring(&mut self, ch: char) {
    self.querystring.push(ch);
  }

  fn rm_last_char_from_querystring(&mut self) {
    self.querystring.pop();
  }

  fn submit_query(&mut self) -> bool {
    // check if valid IP else return false
    if self.apptheme.ipregex.is_match(&self.querystring) {
      self.command_tx.clone().unwrap().send(Action::SubmitQuery(self.querystring.clone())).unwrap_or_else(|err|{
        println!("Error submitting query from Home {}", err);
      });
      return true;
    }
    false
  }

  fn popup_ban(&mut self)-> impl Widget + '_ {

    let querycursor = self.anim_querycursor.state.selected().unwrap();
    let querycursor = self.anim_querycursor.keyframes[querycursor];
    self.ipstring = String::from("");
    let selected_ip: String;
    let sel_idx = self.iplist.state.selected();
    if !self.iplist.items.is_empty() {
      let sel_idx = sel_idx.unwrap();
      selected_ip = self.iplist.items[sel_idx].clone().IP.ip;
      self.ipstring = selected_ip;
    }

    let mut querytext: Vec<Line> = vec![];
    let queryline =   Line::from(vec![
      Span::styled(format!("Ban: {}", self.ipstring), self.apptheme.default_text_style) , 
      Span::styled(querycursor, self.apptheme.fail2ban_bg)
      ]);
    //queryline.patch_style(self.apptheme.selected_ip_bg);
    querytext.push(queryline);
    let mut queryerror =   Line::from(format!("Status: {}", self.iperror));
    queryerror.patch_style(self.apptheme.default_background);
    querytext.push(queryerror);

    let querybox = Paragraph::new(querytext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Ban"));
    querybox

  }

  fn popup_unban(&mut self)-> impl Widget + '_ {

    let querycursor = self.anim_querycursor.state.selected().unwrap();
    let querycursor = self.anim_querycursor.keyframes[querycursor];
    self.ipstring = String::from("");
    let selected_ip: String;
    let sel_idx = self.iplist.state.selected();
    if !self.iplist.items.is_empty() {
      let sel_idx = sel_idx.unwrap();
      selected_ip = self.iplist.items[sel_idx].clone().IP.ip;
      self.ipstring = selected_ip;
    }


    let mut querytext: Vec<Line> = vec![];
    let queryline =   Line::from(vec![
      Span::styled(format!("Unban: {}", self.ipstring), self.apptheme.default_text_style) , 
      Span::styled(querycursor, self.apptheme.fail2ban_bg)
      ]);
    //queryline.patch_style(self.apptheme.selected_ip_bg);
    querytext.push(queryline);

    let mut queryerror =   Line::from(format!("Status: {}", self.iperror));
    queryerror.patch_style(self.apptheme.default_background);
    querytext.push(queryerror);

    let querybox = Paragraph::new(querytext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Unban"));
    querybox

  }

  fn add_to_ipstring(&mut self, ch: char) {
    self.ipstring.push(ch);
  }

  fn rm_last_char_from_ipstring(&mut self) {
    self.ipstring.pop();
  }

  fn submit_ip(&mut self, is_ban:bool) -> bool {
    // check if valid IP else return false
    if self.apptheme.ipregex.is_match(&self.ipstring) {
      if is_ban {
        self.command_tx.clone().unwrap().send(Action::RequestBan).unwrap_or_else(|err|{
          println!("Error submitting query from Home {}", err);
        });
      } else {
        self.command_tx.clone().unwrap().send(Action::RequestUnban).unwrap_or_else(|err|{
          println!("Error submitting query from Home {}", err);
        });
      }

      return true;
    }
    false
  }

  fn toggle_f2bwatcher(&mut self, action_idx: usize) -> Action {
    // check if is active
    if self.f2brunning {
      self.available_actions.items[action_idx].1 = String::from("inactive");
      Action::StopF2BWatcher
    } else {
      self.available_actions.items[action_idx].1 = String::from("active");
      self.f2brunning = true;
      Action::StartF2BWatcher                 
    }
  }

  fn toggle_jctlwatcher(&mut self, action_idx: usize) -> Action {
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
  }

  fn clear_lists(&mut self) {
    self.iplist.items = vec![];
    self.stored_styled_iostreamed.items = vec![];
  }

  fn set_io_capacity_to(&mut self, new_capacity:usize) {
    self.iostreamed_capacity = new_capacity;
    self.stored_styled_iostreamed.trim_to_length(self.iostreamed_capacity);
  }

  fn add_to_capacitystring(&mut self, ch: char) {
    self.iostreamed_capacity_input.push(ch);
  }

  fn rm_last_char_from_capacitystring(&mut self) {
    self.iostreamed_capacity_input.pop();
  }

  fn submit_capacity(&mut self) -> bool {
    // check if valid IP else return false
    let mut new_capacity = self.iostreamed_capacity_input.clone().parse::<usize>().unwrap_or(0);
    if new_capacity > 0 {
      if new_capacity > 10000 {new_capacity = 10000;}
      self.set_io_capacity_to(new_capacity);
      self.iostreamed_capacity_input = String::from("");
      return true;
    }
    false
  }

  fn select_new_theme(&mut self, theme_name: String) {
    for themecontainer in self.available_themes.theme_collection.clone() {
      if themecontainer.name == theme_name {
        self.apptheme = themecontainer.theme;
        break;
      }
    }
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
    // Do matching of general keychars first to deduplicate
    // dont handle key events until we are fully loaded and not showing stats
    let mut action: Action = Action::Blank;
    if self.startup_complete && !self.showing_stats { // fully loaded
      match key.code {
        KeyCode::Esc => return Ok(Some(Action::Quit)),
        KeyCode::Char(keychar) => {
          match keychar {
            // General Hotkeys
            'W'|'w' => {if self.displaymode == DisplayMode::Help {self.displaymode = DisplayMode::Normal;} else {self.displaymode = DisplayMode::Help;} return Ok(Some(Action::Blank))},
            'Q'|'q' => {if self.displaymode == DisplayMode::Query {return Ok(Some(Action::ExitQuery))} else {return Ok(Some(Action::EnterQuery))} },
            'E'|'e' => {return Ok(Some(Action::StatsShow))},
            'C'|'c' => {return Ok(Some(Action::ConfirmClearLists)) },
            'B'|'b' => {if self.displaymode == DisplayMode::Ban {return Ok(Some(Action::ExitBan))} else {return Ok(Some(Action::EnterBan))}},
            'U'|'u' => {if self.displaymode == DisplayMode::Unban {return Ok(Some(Action::ExitUnban))} else {return Ok(Some(Action::EnterUnban))}},
            // DrawMode switching
            'A'|'a' => {self.drawmode = DrawMode::All; return Ok(Some(Action::Blank))},
            'S'|'s' => {self.drawmode = DrawMode::Sticky; return Ok(Some(Action::Blank))},
            'D'|'d' => {self.drawmode = DrawMode::Decaying; return Ok(Some(Action::Blank))},
            // IO-ListNavigation
            'J'|'j' => {self.last_mode = self.mode; return Ok(Some(Action::LogsPrevious))}, // up // LogsSchedulePrevious Schedule locks me out with resetting modes
            'H'|'h' => {self.last_mode = self.mode; return Ok(Some(Action::LogsFirst))}, // top
            'K'|'k' => {self.last_mode = self.mode; return Ok(Some(Action::LogsNext))},  // down
            'L'|'l' => {self.last_mode = self.mode; return Ok(Some(Action::LogsLast))}, // bottom
            'M'|'m' => {self.stored_styled_iostreamed.unselect(); return Ok(Some(Action::Blank))}, // unselect
            '+'|'-' => { return Ok(Some(Action::SetCapacity))}, // unselect
            // IP & Action Navigation
            // -- ArrowKeys
            // IOMode switching
            'F'|'f' => {self.iomode = IOMode::Follow; return Ok(Some(Action::Blank))},
            'G'|'g' => {self.iomode = IOMode::Static; return Ok(Some(Action::Blank))},
            _ => {}
          }
        },
        _ => {},
      };
      action = match self.mode {
        Mode::Processing => return Ok(None),
        Mode::Normal => {
          match key.code {
            KeyCode::Down => {Action::IPsNext},
            KeyCode::Up => {Action::IPsPrevious},
            KeyCode::Left => {
              self.iplist.unselect();
              self.selected_ip = "".to_string();
              //Action::Render
              Action::Blank
            },
            KeyCode::Right | KeyCode::Tab | KeyCode::Enter => {Action::EnterTakeAction},
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
            KeyCode::Right | KeyCode::Enter => {
              let action_idx = self.available_actions.state.selected().unwrap();
              match self.available_actions.items[action_idx].0 {
                "Ban" => {if self.displaymode == DisplayMode::Ban {return Ok(Some(Action::ExitBan))} else {return Ok(Some(Action::EnterBan))}},
                "Unban" => {if self.displaymode == DisplayMode::Unban {return Ok(Some(Action::ExitUnban))} else {return Ok(Some(Action::EnterUnban))}},
                "monitor-fail2ban" => {self.toggle_f2bwatcher(action_idx)},
                "monitor-journalctl" => {self.toggle_jctlwatcher(action_idx)},
                "Stats" => {return Ok(Some(Action::StatsShow))},
                "Query" => {if self.displaymode == DisplayMode::Query {Action::ExitQuery} else {Action::EnterQuery}},
                "Help" => {if self.displaymode == DisplayMode::Help {self.displaymode = DisplayMode::Normal;} else {self.displaymode = DisplayMode::Help;} Action::Blank},
                "Exit" => {Action::Quit},
                _ => {Action::Blank},
              }
            }, 
            _ => {
              self.input.handle_event(&crossterm::event::Event::Key(key));
              //Action::Render
              Action::Blank
            },
          }
        },
        ////
        Mode::Query => {
          match key.code {
            KeyCode::Tab => {self.displaymode = DisplayMode::Normal; 
            match self.last_mode {
              Mode::Normal => {Action::EnterNormal},
              Mode::TakeAction => {Action::EnterTakeAction},
              Mode::Processing => {Action::EnterNormal},
              _ => {Action::EnterNormal},
            }}
            KeyCode::Char(keychar) => {
              match keychar {
                // Digits & Dot
                '1' => {self.add_to_querystring('1'); Action::Render}, // Action render makes it feel way more responsive
                '2' => {self.add_to_querystring('2'); Action::Render},
                '3' => {self.add_to_querystring('3'); Action::Render},
                '4' => {self.add_to_querystring('4'); Action::Render}, 
                '5' => {self.add_to_querystring('5'); Action::Render}, 
                '6' => {self.add_to_querystring('6'); Action::Render}, 
                '7' => {self.add_to_querystring('7'); Action::Render}, 
                '8' => {self.add_to_querystring('8'); Action::Render}, 
                '9' => {self.add_to_querystring('9'); Action::Render},
                '0' => {self.add_to_querystring('0'); Action::Render},
                '.' => {self.add_to_querystring('.'); Action::Render},
                _ => {//Action::Render
                  Action::Blank}
              }
            },
            KeyCode::Backspace => {self.rm_last_char_from_querystring(); Action::Render},
            KeyCode::Enter => {if self.submit_query() {Action::Blank} else {Action::InvalidQuery}}, // print something to the querybox, best -> mark invalid chars / num chars
            _ => {
              self.input.handle_event(&crossterm::event::Event::Key(key));
              //Action::Render
              Action::Blank
            },
          }      
        },
        Mode::ConfirmClear => {
          match key.code {
            KeyCode::Char(keychar) => {
              match keychar {
                // Digits & Dot
                'Y'|'y' => {Action::ConfirmedClearLists}, // Action render makes it feel way more responsive
                'N'|'n' => {Action::AbortClearLists},
                'C'|'c' => {Action::AbortClearLists},
                _ => {//Action::Render
                  Action::Blank}
              }
            },
            KeyCode::Backspace => {Action::AbortClearLists}
            _ => {
              self.input.handle_event(&crossterm::event::Event::Key(key));
              //Action::Render
              Action::Blank            
            },
          }
        },
        ///////
        Mode::SetIOCapacity => {
          match key.code { 
            KeyCode::Char(keychar) => {
              match keychar {
                // Digits
                '1' => {self.add_to_capacitystring('1'); Action::Render}, // Action render makes it feel way more responsive
                '2' => {self.add_to_capacitystring('2'); Action::Render},
                '3' => {self.add_to_capacitystring('3'); Action::Render},
                '4' => {self.add_to_capacitystring('4'); Action::Render}, 
                '5' => {self.add_to_capacitystring('5'); Action::Render}, 
                '6' => {self.add_to_capacitystring('6'); Action::Render}, 
                '7' => {self.add_to_capacitystring('7'); Action::Render}, 
                '8' => {self.add_to_capacitystring('8'); Action::Render}, 
                '9' => {self.add_to_capacitystring('9'); Action::Render},
                '0' => {self.add_to_capacitystring('0'); Action::Render},
                '+'|'-' => {Action::SubmittedCapacity}
                _ => {//Action::Render
                  Action::Blank}
              }
            },          
            KeyCode::Backspace => {self.rm_last_char_from_capacitystring(); Action::Render},
            KeyCode::Enter => {if self.submit_capacity() {Action::SubmittedCapacity} else {self.iostreamed_capacity_input = String::from(""); Action::Blank}},
            _ => {self.input.handle_event(&crossterm::event::Event::Key(key)); Action::Blank  }
          }
        },
        ///////
        Mode::Ban => {  match key.code {
          KeyCode::Tab => {self.displaymode = DisplayMode::Normal; 
          match self.last_mode {
            Mode::Normal => {Action::EnterNormal},
            Mode::TakeAction => {Action::EnterTakeAction},
            Mode::Processing => {Action::EnterNormal},
            _ => {Action::EnterNormal},
          }}
          KeyCode::Char(keychar) => {
            match keychar {
              // Digits & Dot
              '1' => {self.add_to_ipstring('1'); Action::Render}, // Action render makes it feel way more responsive
              '2' => {self.add_to_ipstring('2'); Action::Render},
              '3' => {self.add_to_ipstring('3'); Action::Render},
              '4' => {self.add_to_ipstring('4'); Action::Render}, 
              '5' => {self.add_to_ipstring('5'); Action::Render}, 
              '6' => {self.add_to_ipstring('6'); Action::Render}, 
              '7' => {self.add_to_ipstring('7'); Action::Render}, 
              '8' => {self.add_to_ipstring('8'); Action::Render}, 
              '9' => {self.add_to_ipstring('9'); Action::Render},
              '0' => {self.add_to_ipstring('0'); Action::Render},
              '.' => {self.add_to_ipstring('.'); Action::Render},
              _ => {//Action::Render
                Action::Blank}
            }
          },
          KeyCode::Backspace => {self.rm_last_char_from_ipstring(); Action::Render},
          KeyCode::Enter => {if self.submit_ip(true) {self.iperror = String::from("Success!"); Action::Blank} else {self.iperror = String::from("Invalid IP"); Action::Blank}}, // print something to the querybox, best -> mark invalid chars / num chars
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            //Action::Render
            Action::Blank
          },
        }      
        },
        Mode::Unban => {  match key.code {
        KeyCode::Tab => {self.displaymode = DisplayMode::Normal; 
        match self.last_mode {
          Mode::Normal => {Action::EnterNormal},
          Mode::TakeAction => {Action::EnterTakeAction},
          Mode::Processing => {Action::EnterNormal},
          _ => {Action::EnterNormal},
        }}
        KeyCode::Char(keychar) => {
          match keychar {
            // Digits & Dot
            '1' => {self.add_to_ipstring('1'); Action::Render}, // Action render makes it feel way more responsive
            '2' => {self.add_to_ipstring('2'); Action::Render},
            '3' => {self.add_to_ipstring('3'); Action::Render},
            '4' => {self.add_to_ipstring('4'); Action::Render}, 
            '5' => {self.add_to_ipstring('5'); Action::Render}, 
            '6' => {self.add_to_ipstring('6'); Action::Render}, 
            '7' => {self.add_to_ipstring('7'); Action::Render}, 
            '8' => {self.add_to_ipstring('8'); Action::Render}, 
            '9' => {self.add_to_ipstring('9'); Action::Render},
            '0' => {self.add_to_ipstring('0'); Action::Render},
            '.' => {self.add_to_ipstring('.'); Action::Render},
            _ => {//Action::Render
              Action::Blank}
          }
        },
        KeyCode::Backspace => {self.rm_last_char_from_ipstring(); Action::Render},
        KeyCode::Enter => {if self.submit_ip(false) {self.iperror = String::from("Success!"); Action::Blank} else {self.iperror = String::from("Invalid IP"); Action::Blank}},
        _ => {
          self.input.handle_event(&crossterm::event::Event::Key(key));
          //Action::Render
          Action::Blank
        },
      }      
        },
      };
    }
   
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {

    match action {
      Action::Tick => {},
      Action::StartupDone => {self.startup_complete = true; self.command_tx.clone().unwrap().send(Action::Refresh)?;}
      Action::EnterNormal => {self.mode = Mode::Normal; self.last_mode = self.mode;},
      Action::EnterTakeAction => {self.mode = Mode::TakeAction; self.last_mode = self.mode;},
      Action::EnterProcessing => { self.mode = Mode::Processing;}, // self.last_mode = self.mode;
      Action::ExitProcessing => {self.mode = self.last_mode;}, // TODO: Last mode, solved? No we have to look into the future to see if we want to change to same again and then forgo that
      Action::EnterQuery => {self.last_mode = self.mode; self.mode = Mode::Query; self.displaymode = DisplayMode::Query;},
      Action::ExitQuery => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::EnterBan => {self.last_mode = self.mode; self.mode = Mode::Ban; self.iperror = String::default(); self.displaymode = DisplayMode::Ban;},
      Action::ExitBan => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::EnterUnban => {self.last_mode = self.mode; self.mode = Mode::Unban; self.iperror = String::default(); self.displaymode = DisplayMode::Unban;},
      Action::ExitUnban => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::InternalLog(x) => {self.internal_logs.items.push(x); self.internal_logs.trim_to_length(10); self.internal_logs.next();},
      Action::StartupGotHome(x) => {
        let lat = x.lat.clone().parse::<f64>().unwrap();
        let lon = x.lon.clone().parse::<f64>().unwrap();
        self.home_lon = lon; self.home_lat = lat;
        self.last_lon = lon; self.last_lat = lat;

        let tx = self.command_tx.clone().unwrap();
        tx.send(Action::InternalLog(format!(" ✓ Got home: {}, {}", x.city, x.country)))?;

      }

      //General
      Action::ConfirmClearLists => {self.last_mode = self.mode; self.mode = Mode::ConfirmClear; self.displaymode = DisplayMode::ConfirmClear;},
      Action::AbortClearLists => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::ConfirmedClearLists => { self.clear_lists(); self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::SetCapacity => {self.last_mode = self.mode; self.mode = Mode::SetIOCapacity; self.displaymode = DisplayMode::SetIOCapacity;},
      Action::SubmittedCapacity => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;},
      Action::SelectTheme(x) => {self.select_new_theme(x)},

      // List Actions
      // -- LOG LIST -- iostreamed
      Action::LogsScheduleNext => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsNext);}, // deprec
      Action::LogsNext => {if self.stored_styled_iostreamed.items.len() > 0 {self.stored_styled_iostreamed.next();}},
      Action::LogsSchedulePrevious => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsPrevious);}, // {list_actions::schedule_previous_loglist(self.command_tx.clone().unwrap());},
      Action::LogsPrevious => {if self.stored_styled_iostreamed.items.len() > 0 {self.stored_styled_iostreamed.previous();}},
      Action::LogsScheduleFirst => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsFirst);}, // deprec
      Action::LogsFirst => {  self.stored_styled_iostreamed.state.select(Some(0)) },
      Action::LogsScheduleLast => {list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::LogsLast);}, // deprec
      Action::LogsLast => {  
        if self.stored_styled_iostreamed.items.len() > 0 {
          let idx = Some(self.stored_styled_iostreamed.items.len() - 1);
          self.stored_styled_iostreamed.state.select(idx);
        }
      },
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
      Action::StoppedJCtlWatcher => {self.jctlrunning = false;},
      Action::IONotify(x) => {self.elapsed_notify += 1;},

      Action::InvalidQuery => {self.queryerror = String::from("Invalid Query!");},
      Action::QueryNotFound(x) => {self.queryerror = format!("IP not found: {}", x);},
      Action::SubmitQuery(x) => {self.querystring = String::from("") ;self.queryerror = format!("Querying IP: {}", x);},

      Action::PassGeo(x,y, z) => {parse_passed_geo(self, x.clone(), y.clone(), z)?;},

      // Stats
      Action::StatsShow => {self.showing_stats = true;},
      Action::StatsHide => {self.showing_stats = false;}

      Action::RequestBan => {
        let sel_ip = self.iplist.state.selected();
        let banip: IP;
        if sel_ip.is_some() {
          banip = self.iplist.items[sel_ip.unwrap()].IP.clone();
          if banip.ip == self.ipstring {
            self.command_tx.clone().unwrap().send(Action::BanIP(banip))?;
          } else {
            let mut _ip = IP::default();
            _ip.ip = self.ipstring.clone();
            self.command_tx.clone().unwrap().send(Action::BanIP(_ip))?;
          }

          
        } else {
          //todo!()
        }
      },
      Action::RequestUnban => {
        let sel_ip = self.iplist.state.selected();
        let banip: IP;
        if sel_ip.is_some() {
          banip = self.iplist.items[sel_ip.unwrap()].IP.clone();
          if banip.ip == self.ipstring {
            self.command_tx.clone().unwrap().send(Action::UnbanIP(banip))?;
          } else {
            let mut _ip = IP::default();
            _ip.ip = self.ipstring.clone();
            self.command_tx.clone().unwrap().send(Action::UnbanIP(_ip))?;
          }
          
        } else {
          //todo!()
        }
      },
      Action::Banned(x) => {
        if x {self.infotext = String::from("BANNED");
          list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::ExitBan);
        }
      },
      Action::Unbanned(x) => {
        if x {self.infotext = String::from("BANNED");}
          list_actions::schedule_generic_action(self.command_tx.clone().unwrap(), Action::ExitUnban);
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    
    if self.startup_complete && !self.showing_stats {

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
  
      let actionlist = ui::create_action_list(self.available_actions.clone(), &self.apptheme, self.mode, self.last_mode, self.selected_ip.clone());
  
      let iplist = ui::create_ip_list(self.iplist.clone(), &self.apptheme, self.mode, self.last_mode);

      let term_w = right_layout[1].width as usize;
  
      let iolist = ui::create_io_list(self.stored_styled_iostreamed.clone(), 
        self.iostreamed_capacity, &self.apptheme, term_w, self.available_actions.clone(), self.selected_ip.clone(), self.elapsed_rticks.clone());
  
      // Draw Map to right_upper = 0
      f.render_widget(self.map_canvas(&right_layout[0]), right_layout[0]);
  
      // Draw Read file to right_lower = 1
      f.render_stateful_widget(iolist, right_layout[1], &mut self.stored_styled_iostreamed.state); // CHANGED 
      // f.render_widget(iolist, right_layout[1]);
      
      f.render_widget(create_internal_logs(self), left_layout[0]);
  
      f.render_stateful_widget(iplist, left_layout[1], &mut self.iplist.state);
  
      f.render_stateful_widget(actionlist, left_layout[2], &mut self.available_actions.state);

      // display popups/overlays
      match self.displaymode {
        DisplayMode::Help => {
          let p_area = centered_rect(f.size(), 35, 45);
          f.render_widget(Clear, p_area);
          f.render_widget(ui::create_help_popup(self),p_area);
          },
        DisplayMode::Query => {
          self.anim_querycursor.next();
          let p_area = centered_rect(f.size(), 20, 7);
          f.render_widget(Clear, p_area);
          f.render_widget(ui::create_query_popup(self),p_area);
        },
        DisplayMode::Stats => {},

        DisplayMode::ConfirmClear => {
          let p_area = centered_rect(f.size(), 20, 5);
          f.render_widget(Clear, p_area);
          f.render_widget(ui::create_clearlist_popup(&self.apptheme),p_area);
        },
        DisplayMode::SetIOCapacity => {
          self.anim_querycursor.next();
          let p_area = centered_rect(f.size(), 20, 5);
          f.render_widget(Clear, p_area);
          f.render_widget(ui::popup_set_io_capacity(self.anim_querycursor.clone(), &self.apptheme, self.iostreamed_capacity_input.clone()) ,p_area);
        },
        DisplayMode::Ban => {
          self.anim_querycursor.next();
          let p_area = centered_rect(f.size(), 20, 7);
          f.render_widget(Clear, p_area);
          f.render_widget(self.popup_ban() ,p_area)
        },
        DisplayMode::Unban => {
          self.anim_querycursor.next();
          let p_area = centered_rect(f.size(), 20, 7);
          f.render_widget(Clear, p_area);
          f.render_widget(self.popup_unban() ,p_area)
        },
        _ => {},
      }

/*       // last keys display
      f.render_widget(
        Block::default()
          .title(
            ratatui::widgets::block::Title::from(format!(
              "{:?}",
              &self.last_events.iter().map(|k| crate::config::key_event_to_string(k)).collect::<Vec<_>>()
            ))
            .alignment(Alignment::Right),
          )
          .title_style(Style::default().add_modifier(Modifier::BOLD)),
        Rect { x: area.x + 1, y: area.height.saturating_sub(1), width: area.width.saturating_sub(2), height: 1 },
      ); */


    } else {
      if !self.showing_stats { // Something should definitly be on screen
        f.render_widget(Paragraph::new("You shouldn't see this, if you keep encountering this problem please create an issue referring to Code: E100"), area);
      }
    }

    Ok(())
  }

}
