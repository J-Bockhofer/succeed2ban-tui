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
  geofetcher, gen_structs::Geodata,
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


#[derive(Default)]
struct StatefulList<T> {
  state: ListState,
  items: Vec<T>,
}

impl<T> StatefulList<T> {
  fn with_items(items: Vec<T>) -> StatefulList<T> {
      StatefulList {
          state: ListState::default(),
          items,
      }
  }

  fn next(&mut self) {
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

  fn previous(&mut self) {
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

  fn unselect(&mut self) {
      self.state.select(None);
  }

  fn trim_to_length(&mut self, max_length: usize) {
    while self.items.len() > max_length {
        self.items.remove(0);
    }
  }
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
  iostreamed: StatefulList<(String, usize)>, // do i need a tuple here? // CHANGED
  //iostreamed: Vec<(String, usize)>,
  iplist: StatefulList<(String, Geodata)>,
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

  //styledio: Vec<ListItem<'a>>,
  styledio: Vec<StyledLine>

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
      ("monitor-systemd", String::from("inactive")),
      ("monitor-fail2ban", String::from("inactive")),
    ]);
    self.last_lat = 53.0416;
    self.last_lon = 8.9433;
    self.home_lat = 53.0416;
    self.home_lon = 8.9433;

    self
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  fn map_canvas(&self) -> impl Widget + '_ {
    canvas::Canvas::default()
        .block(Block::default().borders(Borders::ALL).title("World"))
        .marker(Marker::Braille)
        .paint(|ctx| {
            ctx.draw(&canvas::Map {
                color: Color::Green,
                resolution: canvas::MapResolution::High,
            });
            // draw line to home
            //x1: self.home_lon,
            //y1: self.home_lat,
            ctx.draw(&canvas::Line {
              x1: self.last_lon + self.last_direction.0 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
              y1: self.last_lat + self.last_direction.1 * map_range((0.,7.), (0.,1.), self.elapsed_frames),
              x2: self.last_lon,
              y2: self.last_lat,
              color:Color::Cyan,
            });
            ctx.draw(&canvas::Circle {
              x: self.last_lon, // lon
              y: self.last_lat, // lat
              radius: self.elapsed_frames,
              color: Color::Red,
            });
            //ctx.print(self.last_lon, self.last_lat, "X".red());
            ctx.print(self.home_lon, self.home_lat, "H".cyan());
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
  }


  // lifetimes out the wazooo
  // pub fn highlight_io<'b>(&'b mut self) where 'b: 'a, {
  // -> Vec<ListItem>

  // avoid the lifetimes by introducing a new struct?
  pub fn highlight_io(&mut self) {

    let iolines: Vec<StyledLine> = self
    .iostreamed
    .items // change stateful list to simple vector CHANGED
    .iter()
    .map(|i| {
        // split regex logic here
        let collected: Vec<&str> = i.0.split("++++").collect();
        let mut thisline: StyledLine = StyledLine::default();

        let lines: Line = Line::default();
        
        for line in collected {
          let mut splitword: &str = "(/%&$§"; // sth super obscure as the default
          let ip_re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
          let results: Vec<&str> = ip_re
            .captures_iter(&line)
            .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
            .collect();
          let mut cip: &str="";
          if !results.is_empty() {
            // assume only left and right side - not multiple ips in one line
            // assume splitword is on left of ip --- lay out of fail2ban sshd
            cip = results[0];
            let ban_re: Regex = Regex::new(r"Ban").unwrap();
            let found_re = Regex::new(r"Found").unwrap();
            if ban_re.is_match(&line) {splitword = "Ban";}
            else if found_re.is_match(&line)  {splitword = "Found";}
            let fparts: Vec<&str> = line.split(cip).collect();
            let sparts: Vec<&str> = fparts[0].split(splitword).collect();

            //let startspan = Span::styled(sparts[0], Style::default().fg(Color::White));

            thisline.words.push((String::from(sparts[0]), Style::default().fg(Color::White)));

            //lines.spans.push(startspan);

            if sparts.len() > 1 {
              // Found or Ban
              //let splitspan = Span::styled(format!("{} ",splitword), Style::default().fg(Color::LightCyan));
              //lines.spans.push(splitspan);
              if splitword == "Found" {
                thisline.words.push((format!("{} ",splitword), Style::default().fg(Color::LightCyan)));
              }
              else {
                // Ban
                thisline.words.push((format!("{} ",splitword), Style::default().fg(Color::LightYellow)));
              }
            }
            if fparts.len() > 1 {
              //let ipspan = Span::styled(cip, Style::default().fg(Color::LightRed));
              //lines.spans.push(ipspan);
              thisline.words.push((String::from(cip), Style::default().fg(Color::LightRed)));
              //let endspan = Span::styled(fparts[1], Style::default().fg(Color::White));
              //lines.spans.push(endspan);
              thisline.words.push((String::from(fparts[1]), Style::default().fg(Color::White)));
            }
          }
          else {
            // result empty, meaning no ip found
            thisline.words.push((String::from(line), Style::default().fg(Color::White)));

          }
        }
        //let lines = vec![Line::from(i.0.as_str())];
        //ListItem::new(lines).style(Style::default().fg(Color::White))
        thisline
    })
    .collect();

    self.styledio = iolines;


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
      Action::IONotify(x) => {
        // CHANGED self.iostreamed.items.push((x.clone(),1));
        self.iostreamed.items.push((x.clone(),1)); // this introduces the extra CPU load
        self.iostreamed.trim_to_length(20);
        self.elapsed_notify += 1;
        
        // call function to split string
        self.highlight_io();
        
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

          self.iplist.items.push((cip, x.clone()));
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
      }
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
          lines.push(
            format!("X - {}", i.1)
                .italic()
                .into(),
          );            
        }


        ListItem::new(lines).style(Style::default().fg(Color::White))
    })
    .collect();

    // Create a List from all list items and highlight the currently selected one
    let actionlist = List::new(av_actions)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.mode {
            Mode::TakeAction => {Style::new().blue()},
            _ => Style::default(),
          })
        .title("Actions"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");


    let ips: Vec<ListItem> = self
    .iplist      // .items
    .items
    .iter()
    .map(|i| {
        let mut lines = vec![Line::from(i.0.as_str())]; // let mut lines = vec![Line::from(i.0)];
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
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.mode {
            Mode::Normal => {Style::new().blue()},
            _ => Style::default(),
          })
        .title("Last IPs"))
        .highlight_style(
            Style::default()
                .fg(Color::Black)
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");


      
      let iolines: Vec<ListItem> = self
        .styledio 
        .iter()
        .map(|i| {

          let mut line: Line = Line::default();
          for word in i.words.clone() {
            let cspan = Span::styled(word.0, word.1); 
            line.spans.push(cspan);
          }

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
      let mut ioactive: bool = false;
      if self.available_actions.items[2].1 == "active" || self.available_actions.items[3].1 == "active" {
        ioactive = true;
      }

      let iolist_title = Line::from(vec![
        Span::styled(" I/O Stream [ ", Style::default().fg(Color::White)),
        Span::styled(animsymbols[self.elapsed_rticks],
          if ioactive {Style::default().fg(Color::Green)} else {Style::default().fg(Color::Red)}),
        Span::styled(" ] ", Style::default().fg(Color::White)),
      ]);
      // Create a List from all list items and highlight the currently selected one
      let iolist = List::new( iolines) //self.styledio.clone()
          .block(Block::default()
            .borders(Borders::ALL)
            .border_style( 
              match self.mode {
                Mode::Normal => {Style::new().white()},
                _ => Style::new().white(),
              })
            .title(iolist_title)
          )
          .highlight_style(
              Style::default()
                  .bg(Color::LightGreen)
                  .add_modifier(Modifier::BOLD),
          )
          .highlight_symbol(">> ");

      
      let infoblock = Paragraph::new(format!("{}--{}",self.infotext.clone(), self.elapsed_notify.to_string()))
        .set_style(Style::new().green())
        .block(Block::default()
        .borders(Borders::ALL)
        .title("Info"));

    // Draw Map to right_upper = 0
    f.render_widget(self.map_canvas(), right_layout[0]);

    // Draw Read file to right_lower = 1
    f.render_stateful_widget(iolist, right_layout[1], &mut self.iostreamed.state); // CHANGED 
    // f.render_widget(iolist, right_layout[1]);
    
    f.render_widget(infoblock, left_layout[0]);

    f.render_stateful_widget(iplist, left_layout[1], &mut self.iplist.state);

    f.render_stateful_widget(actionlist, left_layout[2], &mut self.available_actions.state);

    Ok(())
  }

  


}

pub fn create_styledio<'a>(home: &'a mut Home<'a>) {

  let iolines: Vec<ListItem> = home
  .iostreamed
  .items // change stateful list to simple vector CHANGED
  .iter()
  .map(|i| {
      // split regex logic here
      let collected: Vec<&str> = i.0.split("++++").collect();
      let mut lines: Line = Line::default();
      
      for line in collected {
        let mut splitword: &str = "(/%&$§";
        let ip_re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
        let results: Vec<&str> = ip_re
          .captures_iter(&line)
          .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
          .collect();
        let mut cip: &str="";
        if !results.is_empty() {
          // assume only left and right side - not multiple ips in one line
          // assume splitword is on left of ip --- lay out of fail2ban sshd
          cip = results[0];
          let ban_re: Regex = Regex::new(r"Ban").unwrap();
          let found_re = Regex::new(r"Found").unwrap();
          if ban_re.is_match(&line) {splitword = "Ban";}
          else if found_re.is_match(&line)  {splitword = "Found";}
          let fparts: Vec<&str> = line.split(cip).collect();
          let sparts: Vec<&str> = fparts[0].split(splitword).collect();

          let startspan = Span::styled(sparts[0], Style::default().fg(Color::White));
          lines.spans.push(startspan);

          if sparts.len() > 1 {
            // Found or Ban
            let splitspan = Span::styled(format!("{} ",splitword), Style::default().fg(Color::LightCyan));
            lines.spans.push(splitspan);
          }
          if fparts.len() > 1 {
            let ipspan = Span::styled(cip, Style::default().fg(Color::LightRed));
            lines.spans.push(ipspan);
            let endspan = Span::styled(fparts[1], Style::default().fg(Color::White));
            lines.spans.push(endspan);
          }
        }
      }
      //let lines = vec![Line::from(i.0.as_str())];
      ListItem::new(lines).style(Style::default().fg(Color::White))
  })
  .collect();

  //home.styledio = iolines;

}