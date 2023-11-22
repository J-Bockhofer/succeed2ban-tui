use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use ratatui::{prelude::*, widgets::*};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;

use super::{Component, Frame};
use crate::{
  action::Action,
  config::{Config, KeyBindings},
};

use tui_input::{backend::crossterm::EventHandler, Input};
use log::error;

#[derive(Default)]
struct StatefulList<T> {
  state: ListState,
  items: Vec<T>,
}

const MAX_LENGTH: usize = 10;

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
  available_actions: StatefulList<(&'a str, usize)>,
  iostreamed: StatefulList<(String, usize)>, // do i need a tuple here?
  pub last_events: Vec<KeyEvent>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub input: Input,
  pub mode: Mode,
}

impl<'a> Home<'a> {
  pub fn new() -> Self {
    Self::default().set_items()
  }

  pub fn set_items(mut self) -> Self {
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
      ("Ban", 1),
      ("Unban", 1),
      ("monitor-systemd", 1),
      ("monitor-fail2ban", 1),
    ]);  
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
/*             ctx.draw(&canvas::Circle {
              x: 8.9433,
              y: 53.0416,
              radius: 2.0,
              color: Color::Red,
            }); */
            ctx.print(8.9433, 53.0416, "X".red());
        })
        .x_bounds([-180.0, 180.0])
        .y_bounds([-90.0, 90.0])
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
            self.items.next(); 
            Action::Render
          },
          KeyCode::Up => {
            self.items.previous();
            Action::Render
          },
          KeyCode::Tab => {
            Action::EnterTakeAction
            
          },
          KeyCode::Enter => {
            if let Some(sender) = &self.command_tx {
              if let Err(e) = sender.send(Action::Quit) {
                error!("Failed to send action: {:?}", e);
              }
            }
            Action::Render
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
          KeyCode::Down => {
            //println!("Arrow Down");
            self.available_actions.next(); 
            Action::Render
          },
          KeyCode::Up => {
            self.available_actions.previous();
            Action::Render
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
        self.iostreamed.items.push((x,1));
        self.iostreamed.trim_to_length(20);
      },
      _ => {},
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, area: Rect) -> Result<()> {
    
    let layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(20), Constraint::Percentage(80)])
      .split(f.size());

/*     let layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(f.size()); */

    let left_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(20), Constraint::Percentage(40), Constraint::Percentage(40)])
      .split(layout[0]);

    let right_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(60), Constraint::Percentage(40)])
      .split(layout[1]);

/*     let sub_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(layout[1]);

    let br_sub_layout = Layout::default()
      .direction(Direction::Horizontal)
      .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
      .split(sub_layout[1]); */
    let av_actions: Vec<ListItem> = self
    .available_actions
    .items
    .iter()
    .map(|i| {
        let mut lines = vec![Line::from(i.0)];
        for _ in 0..i.1 {
            lines.push(
                "X"
                    .italic()
                    .into(),
            );
        }
        ListItem::new(lines).style(Style::default().fg(Color::Red))
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
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");


    let items: Vec<ListItem> = self
    .items
    .items
    .iter()
    .map(|i| {
        let mut lines = vec![Line::from(i.0)];
        for _ in 0..i.1 {
            lines.push(
                "X"
                    .italic()
                    .into(),
            );
        }
        ListItem::new(lines).style(Style::default().fg(Color::Red))
    })
    .collect();

    // Create a List from all list items and highlight the currently selected one
    let itemlist = List::new(items)
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
                .bg(Color::LightGreen)
                .add_modifier(Modifier::BOLD),
        )
        .highlight_symbol(">> ");


      let iolines: Vec<ListItem> = self
      .iostreamed
      .items
      .iter()
      .map(|i| {
          let lines = vec![Line::from(i.0.as_str())];
          ListItem::new(lines).style(Style::default().fg(Color::Red))
      })
      .collect();
  
      // Create a List from all list items and highlight the currently selected one
      let iolist = List::new(iolines)
          .block(Block::default()
          .borders(Borders::ALL)
          .border_style( 
            match self.mode {
              Mode::Normal => {Style::new().blue()},
              _ => Style::default(),
            })
          .title("I/O Stream"))
          .highlight_style(
              Style::default()
                  .bg(Color::LightGreen)
                  .add_modifier(Modifier::BOLD),
          )
          .highlight_symbol(">> ");


    // Draw Map to right_upper = 0
    f.render_widget(self.map_canvas(), right_layout[0]);

    // Draw Read file to right_lower = 1
    f.render_stateful_widget(iolist, right_layout[1], &mut self.iostreamed.state);
    
    f.render_widget(Block::new().borders(Borders::ALL).title("Info"), left_layout[0]);

    f.render_stateful_widget(itemlist, left_layout[1], &mut self.items.state);

    f.render_stateful_widget(actionlist, left_layout[2], &mut self.available_actions.state);

    Ok(())
  }

  


}

