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
  geofetcher,
};

use tui_input::{backend::crossterm::EventHandler, Input};
use log::error;

use regex::Regex;

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
  iplist: StatefulList<(String, String)>,
  pub last_events: Vec<KeyEvent>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub input: Input,
  pub mode: Mode,
  elapsed_rticks: usize,
  selected_ip: String,
  last_lat: f64,
  last_lon: f64,

  home_lat: f64,
  home_lon: f64,

  infotext: String,
  elapsed_notify: usize,

  styledio: Vec<ListItem<'a>>,

}

impl<'a> Home<'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  pub fn set_items(mut self) -> Self {
    self.iplist = StatefulList::with_items(vec![
      (String::from("789.555.555.555"), String::from("Hell")),
      (String::from("160.203.44.55"), String::from("Berlin")),
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
      ("monitor-fail2ban", String::from("active")),
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
            ctx.draw(&canvas::Line {
              x1: self.home_lon,
              y1: self.home_lat,
              x2: self.last_lon,
              y2: self.last_lat,
              color:Color::LightMagenta,
            });
/*             ctx.draw(&canvas::Circle {
              x: 8.9433, // lon
              y: 53.0416, // lat
              radius: 2.0,
              color: Color::Red,
            }); */
            ctx.print(self.last_lon, self.last_lat, "X".red());
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
  }

  pub fn fetchgeo(&self, ip:&str) {

/*     tokio::spawn({
      let geodat = geofetcher::fetch_geolocation(ip).await.unwrap_or(serde_json::Value::default());
    }); */
    


    //println!("{:?}", geodat);
    //let geolat = geodat.get("lat").unwrap();
    //let geolon = geodat.get("lon").unwrap();
    //let geoisp = geodat.get("isp").unwrap();
  }

  // lifetimes out the wazooo
  // pub fn highlight_io<'b>(&'b mut self) where 'b: 'a, {

  pub fn highlight_io(&mut self) -> Vec<ListItem> {

    let iolines: Vec<ListItem> = self
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
            let ban_re = Regex::new(r"Ban").unwrap();
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

    //self.styledio = iolines;

    iolines

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
            self.iplist.next();
            let sel_idx = self.iplist.state.selected().unwrap();
            self.selected_ip = self.iplist.items[sel_idx].0.clone();
            Action::Render
          },
          KeyCode::Up => {
            self.iplist.previous();
            let sel_idx = self.iplist.state.selected().unwrap();
            self.selected_ip = self.iplist.items[sel_idx].0.clone();
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
              _ => {Action::Render},
            }
          },        
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

        
        // call function to split string
        let styledio = self.highlight_io();
/*         for item in styledio {
          self.styledio.push(item.clone());
        } */

        //self.styledio = styledio.clone();
      
        self.elapsed_notify += 1;
      },
      Action::GotGeo(x) => {

        let cip = x.ip;
        let city = x.city;
        let country = x.country;
        let country_code = x.country_code;
        let isp = x.isp;
        let region_name = x.region_name;

        let geolat = x.lat;
        let geolon = x.lon;

        self.last_lat = geolat.parse::<f64>().unwrap();
        self.last_lon = geolon.parse::<f64>().unwrap();

        let cipvec = self.iplist.items.clone();

        if !cipvec.iter().any(|i| i.0==cip) {

          self.iplist.items.push((cip, city));
          self.iplist.trim_to_length(10);

        }        
      },
      Action::Ban => {
        let sel_ip = self.iplist.state.selected();
        let banip: String;
        if sel_ip.is_some() {
          banip = self.iplist.items[sel_ip.unwrap()].1.clone();
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
    const ANIMTICKS: usize = 4;
    let animsymbols = vec!["|","/","―","\\"];

    if self.elapsed_rticks >= ANIMTICKS {
      self.elapsed_rticks = 0;
      //self.infotext = String::from("");
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
          format!("X - {}", i.1)
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

      // Widget for IO STREAM
      // string highlighting function got made here... and is borked
      let iolines: Vec<ListItem> = self
        .iostreamed
        .items // change stateful list to simple vector CHANGED
        .iter()
        .map(|i| {
          ListItem::new(Line::from(i.0.as_str())).style(Style::default().fg(Color::White))
        })
        .collect();
  



      let iolist_title = Line::from(vec![
        Span::styled(" I/O Stream [ ", Style::default().fg(Color::White)),
        Span::styled(animsymbols[self.elapsed_rticks],Style::default().fg(Color::Green)),
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

