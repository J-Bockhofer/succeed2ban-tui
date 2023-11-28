use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use futures::{TryFutureExt, FutureExt};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;


use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
  geofetcher, gen_structs::Geodata, gen_structs::StatefulList,
  themes,
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

#[derive(Default)]
pub struct Home<'a> {
  command_tx: Option<UnboundedSender<Action>>,
  config: Config,
  items: StatefulList<(&'a str, usize)>,
  available_actions: StatefulList<(&'a str, String)>,
  //iostreamed: StatefulList<(String, usize)>, // do i need a tuple here? // CHANGED
  //iostreamed: Vec<(String, usize)>,
  iplist: StatefulList<(String, Geodata, String)>,
  pub last_events: Vec<KeyEvent>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub input: Input,
  pub mode: Mode,
  elapsed_rticks: usize,
  elapsed_frames: f64,

  selected_ip: String,
  last_lat: f64,
  last_lon: f64,

  home_lat: f64,
  home_lon: f64,

  last_direction: (f64, f64), //tuple vector2D that points towards home; 0/x = lon, 1/y = lat
  // home - last 

  infotext: String,
  elapsed_notify: usize,
  debug_me: String,

  //styledio: Vec<ListItem<'a>>,
  //styledio: Vec<StyledLine>,

  //stored_styled_lines: Vec<StyledLine>,

  stored_styled_iostreamed: StatefulList<(StyledLine, String)>,

  apptheme: themes::Theme,

  jctlrunning: bool,
  f2brunning: bool,

  last_username: String,

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
    

    self
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  fn map_canvas(&self, area: &Rect) -> impl Widget + '_ {

    let w = f64::from(area.width.clone());
    let h = f64::from(area.height.clone());

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
            // draw line to home
            ctx.draw(&canvas::Line {
              x1: self.home_lon,
              y1: self.home_lat,
              x2: self.last_lon,
              y2: self.last_lat,
              color:self.apptheme.colors.accent_dblue,
            }); 

            ctx.draw(&canvas::Line {
              x1: self.last_lon + self.last_direction.0 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
              y1: self.last_lat + self.last_direction.1 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
              x2: self.last_lon,
              y2: self.last_lat,
              color: self.apptheme.colors.accent_blue,
            });
           
            ctx.draw(&canvas::Circle {
              x: self.last_lon, // lon
              y: self.last_lat, // lat
              radius: self.elapsed_frames,
              color: self.apptheme.colors.accent_wred,
            });
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

            thisline.words.push((held_unstyled_words.join(" "), self.apptheme.default_text_style));
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
      self.stored_styled_iostreamed.items.push((thisline, last_io.clone()));
      self.stored_styled_iostreamed.trim_to_length(20);

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
  
              self.last_direction = (self.home_lon - self.last_lon, self.home_lat - self.last_lat);
            }


            Action::Render
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
            }
            Action::Render
          },
          KeyCode::Right => {
            Action::EnterTakeAction
          },
          KeyCode::Left => {
            self.iplist.unselect();
            Action::Render
          },
          KeyCode::Tab => {
            Action::EnterTakeAction
            
          },
          KeyCode::Enter => {
            Action::EnterTakeAction
          },
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            Action::Render
          },
        }}
      Mode::TakeAction => {
        match key.code {
          KeyCode::Tab => {
            Action::EnterNormal
          },
          KeyCode::Left => {
            Action::EnterNormal
          },
          KeyCode::Down => {
            //println!("Arrow Down");

            self.available_actions.next(); 

            
            Action::Render

          },
          KeyCode::Up => {

            self.available_actions.previous();

            Action::Render
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
              _ => {Action::Render},
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
              _ => {Action::Render},
            }
          }       
          _ => {
            self.input.handle_event(&crossterm::event::Event::Key(key));
            Action::Render
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
      Action::EnterNormal => {self.mode = Mode::Normal;},
      Action::EnterTakeAction => {self.mode = Mode::TakeAction;},
      Action::EnterProcessing => {self.mode = Mode::Processing;},
      Action::ExitProcessing => {self.mode = Mode::Normal;},
      Action::StoppedJCtlWatcher => {self.jctlrunning = false;},
      Action::IONotify(x) => {

        self.elapsed_notify += 1;
        self.style_incoming_message(x.clone());
        
      },
      Action::GotGeo(x) => {

        let cip = x.ip.clone();
        //let city = x.city;
        //let country = x.country;
        //let country_code = x.country_code;
        //let isp = x.isp;
        //let region_name = x.region_name;

        let geolat = x.lat.clone();
        let geolon = x.lon.clone();

        self.last_lat = geolat.parse::<f64>().unwrap();
        self.last_lon = geolon.parse::<f64>().unwrap();

        self.last_direction = (self.home_lon - self.last_lon, self.home_lat - self.last_lat);

        let cipvec = self.iplist.items.clone();

        if !cipvec.iter().any(|i| i.0==cip) {

          self.iplist.items.push((cip, x.clone(), self.last_username.clone()));
          self.iplist.trim_to_length(10);
          if self.iplist.items.len() > 1 {
            self.iplist.state.select(Option::Some(self.iplist.items.len()-1));
          }
          else {
            self.iplist.state.select(Option::Some(0));
          }


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
        .bg(self.apptheme.colors.default_background)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.mode {
            Mode::TakeAction => {self.apptheme.active_border_style},
            _ => self.apptheme.border_style,
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
        lines.push(
          format!("X - {}, {}", i.1.city, i.1.country)
              .italic()
              .into(),
        );
        ListItem::new(lines).style(Style::default().fg(Color::White))
    })
    .collect();

    // Create a List from all list items and highlight the currently selected one
    let iplist = List::new(ips)
        .bg(self.apptheme.colors.default_background)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.mode {
            Mode::Normal => {self.apptheme.active_border_style},
            _ => self.apptheme.border_style,
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

          let bg_style: Style;
          if i.1 == "Journal" {

            bg_style = self.apptheme.journal_bg;
          } else {
            bg_style = self.apptheme.fail2ban_bg;
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
            .bg(self.apptheme.colors.default_background)
            .borders(Borders::ALL)
            .border_style( 
              match self.mode {
                Mode::Normal => {Style::new().white()},
                _ => Style::new().white(),
              })
            .title(iolist_title)
          )
          .highlight_style(self.apptheme.highlight_item_style)
          .highlight_symbol(">> ");

      
      let infoblock = Paragraph::new(format!("{}-numIn-{}-numLinesLast-{}",self.infotext.clone(), self.elapsed_notify.to_string(), self.debug_me))
        .set_style(Style::default())
        .block(Block::default()
        .bg(self.apptheme.colors.default_background)
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

    Ok(())
  }

  


}

