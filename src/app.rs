use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use futures::channel::mpsc::UnboundedSender;
use notify::{INotifyWatcher, RecommendedWatcher, Watcher};
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
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
  migrations::schema::ip::IP,
};

use regex::Regex;


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
      f2b_cancellation_token: CancellationToken::default(),
      f2b_watcher: Option::None,
      jctl_cancellation_token: CancellationToken::default(),

    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    let mut jctl_sender: Option<mpsc::UnboundedSender<bool>>= Option::None;

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
          Action::StartupConnect => {
             
          },

          Action::StartF2BWatcher => {
              // start the fail2ban watcher
              let action_tx2 = action_tx.clone();
              let action_tx3 = action_tx.clone();
              let action_tx4 = action_tx.clone();



              self.f2b_cancellation_token.cancel();
              tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // make sure we're wound down
              let token = CancellationToken::new();

              
              let _f2b_cancellation_token = token.child_token();
              self.f2b_cancellation_token = token;
              let path = String::from("/var/log/fail2ban.log"); 

              
              // set up watcher
              let (_tx, _rx) = std::sync::mpsc::channel();
              let mut watcher: notify::INotifyWatcher = notify::RecommendedWatcher::new(_tx, notify::Config::default())?;
              watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;


              let filewatcher = tokio::spawn(async move  {
                let _res = tasks::notify_change(&path, action_tx2, _rx);
                  tokio::select! {
                    _ = _f2b_cancellation_token.cancelled() => {
                      drop(watcher);
                      //_res.abort();
                      todo!("Got dropped, so all should be fine");
                      //return;
                    }
                    _ = _res => {

                      todo!("Notify Change ended before it was cancelled, should not happen")
                    }
                  }

                });

                let fetchmsg = format!(" ✔ STARTED fail2ban watcher");
                action_tx4.send(Action::InternalLog(fetchmsg)).expect("LOG: StartF2BWatcher message failed to send");
            },

          Action::StopF2BWatcher => {
            self.f2b_cancellation_token.cancel();

            if self.f2b_cancellation_token.is_cancelled() {
              //todo!("Implement Action::StoppedF2BWatcher to inform Home")
            } else {
              std::thread::sleep(std::time::Duration::from_millis(10));
              action_tx.clone().send(Action::StopF2BWatcher).unwrap();
            }
            let fetchmsg = format!(" ❌ STOPPED fail2ban watcher");
            action_tx.clone().send(Action::InternalLog(fetchmsg)).expect("LOG: StopF2BWatcher message failed to send");
 /*            if let Some(handle) = self.f2bw_handle.take()  {
              // should be more graceful
              
              handle.abort();

              let mut counter = 0;
              while !handle.is_finished() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                counter += 1;
                if counter > 50 {
                  handle.abort();
                }
                if counter > 100 {
                  log::error!("Failed to abort task in 100 milliseconds for unknown reason");
                  break;
                }
              }


            } */

          },
          Action::StartJCtlWatcher => {
            if jctl_sender.is_none() {

              // create sender and receiver for cancellation:
              let (cancel_tx, cancel_rx) = tokio::sync::mpsc::unbounded_channel::<bool>();
              jctl_sender = Option::Some(cancel_tx);

              self.jctl_cancellation_token.cancel();
              tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // make sure we're wound down
              let token = CancellationToken::new();

              
              let _f2b_cancellation_token = token.child_token();
              self.jctl_cancellation_token = token;

              // start the fail2ban watcher
              let action_tx2 = action_tx.clone();
              let action_tx3 = action_tx.clone();

              let journalwatcher = tokio::spawn(async move  {
                let _resp = tasks::monitor_journalctl( action_tx2, cancel_rx);
                tokio::select! {
                  _ = _f2b_cancellation_token.cancelled() => {
                    todo!("Got dropped, so all should be fine");
                  },
                  _ = _resp => {
                    todo!("Journalctl Watcher ended before it was cancelled, should not happen")
                  },
                }

                  
/*                   let _resp = tasks::monitor_journalctl( action_tx2, cancel_rx).await.unwrap_or_else(|err| {
                    action_tx3.send(Action::Error(String::from("Bad Error!"))).unwrap();
                  }); */
                });
              self.jctl_handle = Option::Some(journalwatcher);
              let fetchmsg = format!(" ✔ STARTED journalctl watcher");
              action_tx.clone().send(Action::InternalLog(fetchmsg)).expect("LOG: StartJCTLWatcher message failed to send");
            }
          },
          Action::StopJCtlWatcher => {
            if let Some(jctl_sender) = jctl_sender.take()  {
              // should be more graceful
              let handle = self.jctl_handle.take().unwrap();
              self.jctl_cancellation_token.cancel();

              jctl_sender.send(false).unwrap_or_else(|err| {
                println!("Failed to send JCTL abort with Error: {}", err);
              });
              handle.abort();
              
              let mut counter = 0;
              while !handle.is_finished() {
                std::thread::sleep(std::time::Duration::from_millis(1));
                counter += 1;
                if counter > 50 {
                  jctl_sender.send(false).unwrap_or_else(|err| {
                    println!("Failed to send JCTL abort with Error: {}", err);
                  });
                  handle.abort();
                }
                if counter > 100 {
                  log::error!("Failed to abort task in 100 milliseconds for unknown reason");
                  break;
                }
              }
              if self.jctl_cancellation_token.is_cancelled() {
                let fetchmsg = format!(" ❌ STOPPED journalctl watcher");
                action_tx.clone().send(Action::InternalLog(fetchmsg)).expect("LOG: StopJCTLWatcher message failed to send");
              }
            }

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
