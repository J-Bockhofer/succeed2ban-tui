use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
//use futures::channel::mpsc::UnboundedSender;
use notify::{Event, INotifyWatcher, RecommendedWatcher, Watcher};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc::UnboundedSender;
use tokio::{sync::mpsc, task::JoinHandle};
use tokio::process::Command;


use rusqlite::{Connection, Result as ConnectionResult};
use tokio::sync::Mutex;
use tokio_util::sync::CancellationToken;
use std::sync::Arc;



use crate::{
  action::Action,
  components::{home::Home, fps::FpsCounter, Component, startup::Startup, stats::Stats},
  config::Config,
  mode::Mode,
  tui,
  tasks,
  geofetcher,
  gen_structs,
  database::schema::ip::IP,
};

use regex::Regex;

mod f2b_watcher;
mod jctl_watcher;

pub struct App {
  pub config: Config,
  pub tick_rate: f64,
  pub frame_rate: f64,
  pub components: Vec<Box<dyn Component>>,
  pub should_quit: bool,
  pub should_suspend: bool,
  pub mode: Mode,
  pub last_tick_key_events: Vec<KeyEvent>,
  pub last_ip: String,
  pub stored_geo: Vec<IP>,
  f2bw_handle: Option<JoinHandle<()>>,
  jctl_handle: Option<JoinHandle<()>>,
  dbconn: Option<Connection>,

  f2b_logpath: String,
  f2b_cancellation_token: CancellationToken,
  f2b_watcher: Option<INotifyWatcher>,
  jctl_cancellation_token: CancellationToken,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let home = Home::new();
    let fps = FpsCounter::default();
    let startup = Startup::new();
    let stats = Stats::new();
    let config = Config::new()?;
    let mode = Mode::Startup;
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(home), Box::new(fps), Box::new(startup), Box::new(stats)], //Box::new(home), Box::new(fps)
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      last_ip: String::new(),
      stored_geo: Vec::new(),
      f2bw_handle: Option::None,
      jctl_handle: Option::None,
      dbconn: Option::None,
      f2b_logpath: String::from("/var/log/fail2ban.log"),
      f2b_cancellation_token: CancellationToken::default(),
      f2b_watcher: Option::None,
      jctl_cancellation_token: CancellationToken::default(),
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();


    let mut tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
    // tui.mouse(true);
    tui.enter()?;

    for component in self.components.iter_mut() {
      component.register_action_handler(action_tx.clone())?;
    }

    for component in self.components.iter_mut() {
      component.register_config_handler(self.config.clone())?;
    }

    for component in self.components.iter_mut() {
      component.init(tui.size()?)?;
    }



    loop {
      if let Some(e) = tui.next().await {
        match e {
          tui::Event::Quit => action_tx.send(Action::Quit)?,
          tui::Event::Tick => action_tx.send(Action::Tick)?,
          tui::Event::Render => action_tx.send(Action::Render)?,
          tui::Event::Resize(x, y) => action_tx.send(Action::Resize(x, y))?,
          tui::Event::Key(key) => {
            if let Some(keymap) = self.config.keybindings.get(&self.mode) {
              if let Some(action) = keymap.get(&vec![key]) {
                log::info!("Got action: {action:?}");
                action_tx.send(action.clone())?;
              } else {
                // If the key was not handled as a single key action,
                // then consider it for multi-key combinations.
                self.last_tick_key_events.push(key);

                // Check for multi-key combinations
                if let Some(action) = keymap.get(&self.last_tick_key_events) {
                  log::info!("Got action: {action:?}");
                  action_tx.send(action.clone())?;
                }
              }
            };
          },
          _ => {},
        }
        for component in self.components.iter_mut() {
          if let Some(action) = component.handle_events(Some(e.clone()))? {
            action_tx.send(action)?;
          }
        }
      }

      while let Ok(action) = action_rx.try_recv() {
        if action != Action::Tick && action != Action::Render {
          log::debug!("{action:?}");
        }
        match action {
          Action::Tick => {
            self.last_tick_key_events.drain(..);
          },
          Action::Quit => self.should_quit = true,
          Action::Suspend => self.should_suspend = true,
          Action::Resume => self.should_suspend = false,
          Action::StartupDone => self.mode = Mode::Home, //
          Action::StatsShow => self.mode = Mode::Stats,
          Action::StatsHide => self.mode = Mode::Home,
          Action::Resize(w, h) => {
            tui.resize(Rect::new(0, 0, w, h))?;
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::Render => {
            tui.draw(|f| {
              for component in self.components.iter_mut() {
                let r = component.draw(f, f.size());
                if let Err(e) = r {
                  action_tx.send(Action::Error(format!("Failed to draw: {:?}", e))).unwrap();
                }
              }
            })?;
          },
          Action::StartF2BWatcher => {
            self.start_f2b_watcher(action_tx.clone()).await?;
          },
          Action::StopF2BWatcher => {
            self.stop_f2b_watcher(action_tx.clone()).await?;
          },
          Action::StartJCtlWatcher => {
            self.start_jctl_watcher(&action_tx).await?;
          },
          Action::StopJCtlWatcher => {
            self.stop_jctl_watcher(&action_tx).await?;
          },
          _ => {},
        }

        for component in self.components.iter_mut() {
          if let Some(action) = component.update(action.clone())? {
            action_tx.send(action)?
          };
        }
      }
      if self.should_suspend {
        tui.suspend()?;
        action_tx.send(Action::Resume)?;
        tui = tui::Tui::new()?.tick_rate(self.tick_rate).frame_rate(self.frame_rate);
        // tui.mouse(true);
        tui.enter()?;
      } else if self.should_quit {
        // cancel running tasks here
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }

}
