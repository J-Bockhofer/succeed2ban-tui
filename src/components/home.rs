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
  animations::Animation,
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



fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
  to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
}



#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  TakeAction,
  Processing,
  Query,
  ConfirmClear,
  SetIOCapacity,
  Ban,
  Unban,
}


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DrawMode {
  #[default]
  Sticky,
  Decaying,
  All,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
  #[default]
  Normal,
  Help,
  Query,
  Stats,
  ConfirmClear,
  SetIOCapacity,
  Ban,
  Unban,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum IOMode {
  #[default]
  Follow, // will jump to freshly received IP
  Static, // will stay at selected IP
}


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

  fn centered_rect(&self, r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ])
      .split(r);
  
    Layout::default()
      .direction(Direction::Horizontal)
      .constraints([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ])
      .split(popup_layout[1])[1]
  }

  fn create_help_popup(&self) -> impl Widget + '_ {
    // make a layout in center of the screen, outside this function, pass area to this  
    let active_drawmode = match self.drawmode {
      DrawMode::All => {"All   "},
      DrawMode::Decaying => {"Decay "},
      DrawMode::Sticky => {"Sticky"},
    };
    let active_iomode = match self.iomode {
      IOMode::Follow => {"Follow"},
      IOMode::Static => {"Static"},
    };

    // make text
    let mut helptext: Vec<Line> = vec![];
    let mut hheader =   Line::from(format!("---           HOTKEYS       ---                                                                 -"));
    hheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(hheader);
    helptext.push(                Line::from(format!("Key:          Name          Info")));
    let mut hheader =   Line::from(format!("---           General       ---                                                                 -"));
    hheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(hheader);    
    helptext.push(                Line::from(format!("Arrowkeys:    Select        Select item in IPs or Actions dependent on mode")));
    helptext.push(                Line::from(format!("Tab:          Mode          Switch Mode between IP-List & Actions")));
    helptext.push(                Line::from(format!("W|w:          Help          Toggle help")));
    helptext.push(                Line::from(format!("Q|q:          Query         Toggle query input for IP data from db")));
    helptext.push(                Line::from(format!("E|e:          Stats         Switch to Stats-Screen")));  
    helptext.push(                Line::from(format!("C|c:          Clear         Clears IP and I/O Lists")));
    helptext.push(                Line::from(format!("Enter:        Execute       Context dependent selection or execution")));
    let mut hheader =   Line::from(format!("---           Drawmode      ---                                           {}                -", active_drawmode)); // for more spaces bc inserted string has six characters
    hheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(hheader);
    helptext.push(                Line::from(format!("A|a:          All           Draws all connections all the time")));
    helptext.push(                Line::from(format!("S|s:          Sticky        Draws only the selection connection")));
    helptext.push(                Line::from(format!("D|d:          Decay         Draws each connection for 10 seconds")));
    let mut ioheader =  Line::from(format!("---           I/O Stream    ---                                 Capacity: {}                -", self.iostreamed_capacity));
    ioheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(ioheader);
    helptext.push(                Line::from(format!("H|h:          First         Select oldest line in I/O Streamed")));
    helptext.push(                Line::from(format!("J|j:          Previous      Select previous line in I/O Streamed")));
    helptext.push(                Line::from(format!("K|k:          Next          Select next line in I/O Streamed")));
    helptext.push(                Line::from(format!("L|l:          Last          Select latest line in I/O Streamed")));
    helptext.push(                Line::from(format!("N|n:          Unselect      Reset line selection in I/O Streamed")));
    helptext.push(                Line::from(format!("+|-:          Set Capacity  Input a new capacity for I/O Streamed")));
    let mut hheader =   Line::from(format!("---           IO-Mode       ---                                           {}                -", active_iomode)); // four more spaces bc inserted string has six characters
    hheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(hheader);
    helptext.push(                Line::from(format!("F|f:          Follow        Auto-selects the last received IP")));
    helptext.push(                Line::from(format!("G|g:          Static        Selection stays where you left it")));   
    let mut hheader =   Line::from(format!("---           DEBUG         ---                                                                 -"));    
    hheader.patch_style(self.apptheme.fail2ban_bg);
    helptext.push(hheader);
    helptext.push(Line::from(format!("Received I/O:   {}", self.infotext.clone())));
    helptext.push(Line::from(format!("numLinesLast:   {}", self.elapsed_notify.to_string())));
    helptext.push(Line::from(format!("dbg-msg:        {}", self.debug_me)));

    let infoblock = Paragraph::new(helptext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Help"));
    infoblock

  }

  fn create_query_popup(&self)-> impl Widget + '_ {

    let querycursor = self.anim_querycursor.state.selected().unwrap();
    let querycursor = self.anim_querycursor.keyframes[querycursor];

    let mut querytext: Vec<Line> = vec![];
    let queryline =   Line::from(vec![
      Span::styled(format!("Query: {}", self.querystring), self.apptheme.selected_ip_bg) , 
      Span::styled(querycursor, self.apptheme.fail2ban_bg)
      ]);
    //queryline.patch_style(self.apptheme.selected_ip_bg);
    querytext.push(queryline);

    let mut queryerror =   Line::from(format!("Status: {}", self.queryerror));
    queryerror.patch_style(self.apptheme.default_background);
    querytext.push(queryerror);

    let querybox = Paragraph::new(querytext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Query"));
    querybox

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

  fn create_clearlist_popup(&self)  -> impl Widget + '_  {

    let mut clearlisttext: Vec<Line> = vec![];
    let clearlistline =   Line::from(vec![
      Span::styled(format!("Press "), self.apptheme.default_text_style), 
      Span::styled(format!("Y | y "), Style::default().fg(self.apptheme.colors.accent_lime)),
      Span::styled(format!("to confirm or "), self.apptheme.default_text_style),
      Span::styled(format!("N | n "), Style::default().fg(self.apptheme.colors.accent_orange)),
      Span::styled(format!("to cancel."), self.apptheme.default_text_style),
      ]);
    //clearlistline.patch_style(self.apptheme.selected_ip_bg);
    clearlisttext.push(clearlistline);

    let clearlistbox = Paragraph::new(clearlisttext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Confirm to clear list"));
    clearlistbox

  }

  fn set_io_capacity_to(&mut self, new_capacity:usize) {
    self.iostreamed_capacity = new_capacity;
  }

  fn popup_set_io_capacity(&self) -> impl Widget + '_ {

    let capacitycursor = self.anim_querycursor.state.selected().unwrap();
    let capacitycursor = self.anim_querycursor.keyframes[capacitycursor];

    let mut capacitytext: Vec<Line> = vec![];
    let capacityline =   Line::from(vec![
      Span::styled(format!("New Capacity: {}", self.iostreamed_capacity_input), self.apptheme.selected_ip_bg) , 
      Span::styled(capacitycursor, self.apptheme.fail2ban_bg)
      ]);
    //capacityline.patch_style(self.apptheme.selected_ip_bg);
    capacitytext.push(capacityline);

    let capacitybox = Paragraph::new(capacitytext)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title("Set I/O Stream capacity"));
    capacitybox
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


  /// Styles the incoming lines from either journalctl or fail2ban log.
  /// Fail2Ban may send a String that contains multiple lines which are delimited by "++++".
  /// Styled lines (saved in self.stored_styled_iostreamed) contain colored substrings / words.
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
      self.stored_styled_iostreamed.trim_to_length(self.iostreamed_capacity);

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
                "Ban" => {Action::RequestBan},
                "monitor-fail2ban" => {self.toggle_f2bwatcher(action_idx)},
                "monitor-journalctl" => {self.toggle_jctlwatcher(action_idx)},
                "Stats" => {Action::Blank}, // TODO
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
      Action::EnterQuery => {self.last_mode = self.mode; self.mode = Mode::Query; self.displaymode = DisplayMode::Query;}
      Action::ExitQuery => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;}
      Action::EnterBan => {self.last_mode = self.mode; self.mode = Mode::Ban; self.iperror = String::default(); self.displaymode = DisplayMode::Ban;}
      Action::ExitBan => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;}
      Action::EnterUnban => {self.last_mode = self.mode; self.mode = Mode::Unban; self.iperror = String::default(); self.displaymode = DisplayMode::Unban;}
      Action::ExitUnban => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;}


      //General
      Action::ConfirmClearLists => {self.last_mode = self.mode; self.mode = Mode::ConfirmClear; self.displaymode = DisplayMode::ConfirmClear;}
      Action::AbortClearLists => {self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;}
      Action::ConfirmedClearLists => { self.clear_lists(); self.mode = self.last_mode; self.displaymode = DisplayMode::Normal;}
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

      Action::GotGeo(x,y, z) => {

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
          self.iplist.trim_to_length(self.iplist_capacity); // change to const

          if self.iomode == IOMode::Follow {
            if self.iplist.items.len() > 1 {
              self.iplist.state.select(Option::Some(self.iplist.items.len()-1));
            }
            else {
              self.iplist.state.select(Option::Some(0));
            }
  
            self.selected_ip = cip;
          }

          
        } 
        else {
          // ip is already in vector, need to select it again if IOmode is follow
          if self.iomode == IOMode::Follow {
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
        }    
      },

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
  
  
          ListItem::new(lines).style(self.apptheme.default_text_style)
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
          ListItem::new(lines).style(self.apptheme.default_text_style)
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
          Span::styled(" I/O Stream [ ", self.apptheme.default_text_style),
          Span::styled(animsymbols[self.elapsed_rticks],
            match ioactive { 0 => {Style::default().fg(self.apptheme.colors.accent_wred)}, 1 => {Style::default().fg(self.apptheme.colors.accent_lpink)}, 2 => {Style::default().fg(self.apptheme.colors.accent_blue)} _ => {Style::default().fg(self.apptheme.colors.accent_wred)}}),
          Span::styled(" ] ", self.apptheme.default_text_style),
        ]);
  
        let iolist_selected_idx = self.stored_styled_iostreamed.state.selected();
        let selected_symb = if iolist_selected_idx.is_some() { let selnum = iolist_selected_idx.unwrap() + 1;
          selnum.to_string()} else {String::from("-")};
        let ciolist_len = self.stored_styled_iostreamed.items.len();
        let list_capacity_diff = self.iostreamed_capacity - ciolist_len; 
        
        let capacity_color = if list_capacity_diff < 10 {self.apptheme.colors.accent_blue} else if list_capacity_diff == 0 {self.apptheme.colors.accent_orange} else {Color::White};


        let iolist_capacity_display = Line::from(vec![
          Span::styled(format!("[ "), self.apptheme.default_text_style),
          Span::styled(format!("{}", selected_symb), Style::default().fg(self.apptheme.colors.accent_blue)), // selected
          Span::styled(format!(" : "), self.apptheme.default_text_style), // separator
          Span::styled(format!("{}", ciolist_len), Style::default().fg(capacity_color)), // current num
          Span::styled(format!(" / ", ), self.apptheme.default_text_style), // separator
          Span::styled(format!("{}", self.iostreamed_capacity), Style::default().fg(capacity_color)), // capacity
          Span::styled(format!(" ]"), self.apptheme.default_text_style),
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
              .title(block::Title::from(iolist_title).alignment(Alignment::Left))
              .title(block::Title::from(iolist_capacity_display).alignment(Alignment::Right))
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

      // display popups/overlays
      match self.displaymode {
        DisplayMode::Help => {
          let p_area = self.centered_rect(f.size(), 35, 45);
          f.render_widget(Clear, p_area);
          f.render_widget(self.create_help_popup(),p_area);
          },
        DisplayMode::Query => {
          self.anim_querycursor.next();
          let p_area = self.centered_rect(f.size(), 20, 7);
          f.render_widget(Clear, p_area);
          f.render_widget(self.create_query_popup(),p_area);
        },
        DisplayMode::Stats => {},

        DisplayMode::ConfirmClear => {
          let p_area = self.centered_rect(f.size(), 20, 5);
          f.render_widget(Clear, p_area);
          f.render_widget(self.create_clearlist_popup(),p_area);
        },
        DisplayMode::SetIOCapacity => {
          self.anim_querycursor.next();
          let p_area = self.centered_rect(f.size(), 20, 5);
          f.render_widget(Clear, p_area);
          f.render_widget(self.popup_set_io_capacity() ,p_area);
        },
        DisplayMode::Ban => {
          self.anim_querycursor.next();
          let p_area = self.centered_rect(f.size(), 20, 7);
          f.render_widget(Clear, p_area);
          f.render_widget(self.popup_ban() ,p_area)
        },
        DisplayMode::Unban => {
          self.anim_querycursor.next();
          let p_area = self.centered_rect(f.size(), 20, 7);
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

