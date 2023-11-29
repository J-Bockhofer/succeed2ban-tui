use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use super::{Component, Frame};
use crate::{action::Action, config::key_event_to_string, themes,};

use rand::prelude::*;

fn map_range(from_range: (f64, f64), to_range: (f64, f64), s: f64) -> f64 {
    to_range.0 + (s - from_range.0) * (to_range.1 - to_range.0) / (from_range.1 - from_range.0)
  }


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
  Loading,
  Done,
  Confirmed,
}

#[derive(Default)]
pub struct Startup {
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
  points: Vec<(f64,f64,f64,f64)>,
}

impl Startup {
  pub fn new() -> Self {
    Self::default()
  }

  pub fn set_rng_points(mut self) -> Self {
    let mut rng = rand::thread_rng();
    let num_lines: usize = rng.gen_range(0..20);
    let mut points: Vec<(f64,f64,f64,f64)> = vec![];
    for _ in 0..num_lines {
        let x: f64 = 0.;//rng.gen_range(-180.0..180.0);
        let y: f64 = 0.;//rng.gen_range(-90.0..90.0);
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

  pub fn tick(&mut self) {
    log::info!("Tick");
    self.num_ticks += 1;
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    log::debug!("Render Tick");
    self.elapsed_frames += 1.;

    if self.elapsed_frames == 1. {
        let mut rng = rand::thread_rng();
        let x: f64 = 0.;//rng.gen_range(-180.0..180.0);
        let y: f64 = 0.;
        let x2: f64 = rng.gen_range(-180.0..180.0);
        let y2: f64 = rng.gen_range(-90.0..90.0);
        self.points.push((x,y,x2,y2));
        if self.points.len() > 20 {
            self.points = vec![];
        }
    }

    if self.elapsed_frames > 12. {
        self.elapsed_frames = 0.;
        if self.num_ticks > 20 {
            self.mode = Mode::Done;
            let _ = self.action_tx.clone().unwrap().send(Action::StartupDone);
        }
    }
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn add(&mut self, s: String) {
    self.text.push(s)
  }

}

impl Component for Startup {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    self.last_events.push(key.clone());
    let action = match self.mode {
      Mode::Normal | Mode::Processing => Action::Render,
      Mode::Done => Action::StartupDone,
      _ =>  {self.input.handle_event(&crossterm::event::Event::Key(key));
      Action::Render},
    };
    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
    match action {
      Action::Tick => {self.tick()},
      Action::Render => self.render_tick(),
      Action::EnterNormal => {
        self.mode = Mode::Normal;
      },
      Action::EnterProcessing => {
        self.mode = Mode::Processing;
      },
      Action::ExitProcessing => {
        // TODO: Make this go to previous mode instead
        self.mode = Mode::Normal;
      },
      _ => (),
    }
    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {


    match self.mode {
        Mode::Loading | Mode::Normal => {





            let layout = Layout::default().constraints([Constraint::Percentage(80), Constraint::Percentage(20)].as_ref()).split(rect);

            let canvas = canvas::Canvas::default()
            .background_color(self.apptheme.colors.default_background)
            .block(Block::default().borders(Borders::ALL).title("World").bg(self.apptheme.colors.default_background))
            .marker(Marker::Braille)
            .paint( |ctx| {

    
                ctx.draw(&canvas::Map {
                    color: self.apptheme.colors.default_map_color,
                    resolution: canvas::MapResolution::High,
                });

                for point in self.points.iter() {

                    let direction = (point.0 - point.2, point.1 - point.3);

                    ctx.draw(&canvas::Line {
                        x1: point.0,
                        y1: point.1,
                        x2: point.2,
                        y2: point.3,
                        color:self.apptheme.colors.accent_dblue,
                      }); 
          
                      ctx.draw(&canvas::Line {
                        x1: point.2 + direction.0 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        y1: point.3 + direction.1 * map_range((0.,11.), (0.,0.9), self.elapsed_frames),
                        x2: point.2,
                        y2: point.3,
                        color: self.apptheme.colors.accent_blue,
                      });                    

                } 


                ctx.draw(&canvas::Circle {
                    x: 0., // lon
                    y: 0., // lat
                    radius:  self.render_ticker as f64,
                    color: self.apptheme.colors.accent_orange,
                  });

            })
            .x_bounds([-180.0, 180.0])
            .y_bounds([-90.0, 90.0]);         


            let mut text: Vec<Line> = self.text.clone().iter().map(|l| Line::from(l.clone())).collect();
            text.insert(0, "".into());
            text.insert(0, "Type into input and hit enter to display here".dim().into());
            text.insert(0, "".into());
            text.insert(0, format!("Render Ticker: {}", self.render_ticker).into());
            text.insert(0, format!("App Ticker: {}", self.app_ticker).into());
            text.insert(0, format!("Counter: {}", self.counter).into());
            text.insert(0, "".into());
            text.insert(
            0,
            Line::from(vec![
                "Press ".into(),
                Span::styled("j", Style::default().fg(Color::Red)),
                " or ".into(),
                Span::styled("k", Style::default().fg(Color::Red)),
                " to ".into(),
                Span::styled("increment", Style::default().fg(Color::Yellow)),
                " or ".into(),
                Span::styled("decrement", Style::default().fg(Color::Yellow)),
                ".".into(),
            ]),
            );
            text.insert(0, "".into());

            f.render_widget(
                Paragraph::new(text)
                  .block(
                    Block::default()
                      .title("Setting up")
                      .title_alignment(Alignment::Center)
                      .borders(Borders::ALL)
                      .border_style(self.apptheme.border_style)
                      .border_type(BorderType::Rounded),
                  )
                  .style(Style::default().fg(self.apptheme.colors.accent_blue).bg(self.apptheme.colors.lblack))
                  .alignment(Alignment::Center),
                layout[1],
              );

            f.render_widget(canvas, layout[0]);


        },
        _ => {},
    }

    Ok(())
  }
}