use color_eyre::eyre::Result;
use crossterm::event::KeyEvent;
use ratatui::prelude::Rect;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, task::JoinHandle};
use tokio::process::Command;

use crate::{
  action::Action,
  components::{home::Home, fps::FpsCounter, Component},
  config::Config,
  mode::Mode,
  tui,
  tasks,
  geofetcher,
  gen_structs::{self, Geodata},
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
  pub stored_geo: Vec<gen_structs::Geodata>,
  f2bw_handle: Option<JoinHandle<()>>,
}

impl App {
  pub fn new(tick_rate: f64, frame_rate: f64) -> Result<Self> {
    let home = Home::new();
    let fps = FpsCounter::default();
    let config = Config::new()?;
    let mode = Mode::Home;
    Ok(Self {
      tick_rate,
      frame_rate,
      components: vec![Box::new(home), Box::new(fps)], //Box::new(home), Box::new(fps)
      should_quit: false,
      should_suspend: false,
      config,
      mode,
      last_tick_key_events: Vec::new(),
      last_ip: String::new(),
      stored_geo: Vec::new(),
      f2bw_handle: Option::None,
    })
  }

  pub async fn run(&mut self) -> Result<()> {
    let (action_tx, mut action_rx) = mpsc::unbounded_channel();

    // try make filereader here?
/*     let action_tx2 = action_tx.clone();
    let action_tx3 = action_tx.clone();


    let _filewatcher = tokio::spawn(async move  {
      
        let path = String::from("/var/log/fail2ban.log"); // easy test "/home/projects/ratui/text.txt" // /var/log/fail2ban.log
        //println!("now running on a worker thread");

        // bog standard polling file reader after
        // https://stackoverflow.com/questions/71632833/how-to-continuously-watch-and-read-file-asynchronously-in-rust-using-tokio
        //let _resp = tasks::follow_file(&path, action_tx2.clone()).await;

        // Notify interface CPU problems
        let _resp = tasks::notify_change(&path, action_tx2.clone()).await.unwrap_or_else(|err| {
          action_tx3.send(Action::Error(String::from("Bad Error!"))).unwrap();
        });
      }); */

/*     let path = String::from("/home/projects/ratui/text.txt");
    let _resp = tasks::notify_change(&path, action_tx2.clone()).await.unwrap_or_else(|err| {
      action_tx3.send(Action::Error(String::from("Bad Error!"))).unwrap();
    });  */   




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
          Action::IONotify(ref x) => {
            let re = Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap();
            let results: Vec<&str> = re
              .captures_iter(x)
              .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
              .collect();
            let cip: &str;

            if !results.is_empty() {
              cip = results[0];
              // string contained an IPv4
              let mut maybe_data: Geodata= Geodata::default();
              let mut is_in_list: bool = false;

              for stored_dat in self.stored_geo.clone() {
                if cip == stored_dat.ip {
                  is_in_list = true;
                  maybe_data = stored_dat;
                  break
                }

              }

              if !is_in_list {
                self.last_ip = String::from(cip);
                let geodat = geofetcher::fetch_geolocation(cip).await.unwrap_or(serde_json::Value::default());
  
                let geoip = String::from(geodat.get("query").unwrap().as_str().unwrap());
                let geolat = geodat.get("lat").unwrap().as_number().unwrap().to_string();
                let geolon = geodat.get("lon").unwrap().as_number().unwrap().to_string();
                let geoisp = String::from(geodat.get("isp").unwrap().as_str().unwrap());
    
                let geocountry = String::from(geodat.get("country").unwrap().as_str().unwrap());
                let geocity = String::from(geodat.get("city").unwrap().as_str().unwrap());
                let geocountrycode = String::from(geodat.get("countryCode").unwrap().as_str().unwrap());
                let georegionname = String::from(geodat.get("regionName").unwrap().as_str().unwrap());
    
    
                let mut geodata = gen_structs::Geodata::new();
                geodata.ip = geoip;
                geodata.lat = geolat;
                geodata.lon = geolon;
                geodata.isp = geoisp;
    
                geodata.country = geocountry;
                geodata.country_code = geocountrycode;
                geodata.city = geocity;
                geodata.region_name = georegionname;
    
                self.stored_geo.push(geodata.clone());

                action_tx.send(Action::GotGeo(geodata))?;
              } else {
                // data is stored
                action_tx.send(Action::GotGeo(maybe_data))?;  
              }

            }

          },
          Action::BanIP(ref x) => {
            
            let response = Command::new("echo")
              .arg("hello")
              .arg("world")
              .output()
              .await
              .expect("failed to spawn");

            // Await until the command completes
            let status = response.status.success();
            //println!("the command exited with: {}", status);
            action_tx.send(Action::Banned(status))?;
            todo!();
          }
          Action::StartF2BWatcher => {
            if self.f2bw_handle.is_none() {
              // start the fail2ban watcher
              let action_tx2 = action_tx.clone();
              let action_tx3 = action_tx.clone();

              let filewatcher = tokio::spawn(async move  {
                
                  let path = String::from("/var/log/fail2ban.log"); // easy test "/home/projects/ratui/text.txt" // /var/log/fail2ban.log
                  //println!("now running on a worker thread");

                  // bog standard polling file reader after
                  // https://stackoverflow.com/questions/71632833/how-to-continuously-watch-and-read-file-asynchronously-in-rust-using-tokio
                  //let _resp = tasks::follow_file(&path, action_tx2.clone()).await;

                  // Notify interface CPU higher but no polling shit and more stuff handled thanks
                  let _resp = tasks::notify_change(&path, action_tx2).await.unwrap_or_else(|err| {
                    action_tx3.send(Action::Error(String::from("Bad Error!"))).unwrap();
                  });
                });
              self.f2bw_handle = Option::Some(filewatcher);
            }


          },
          Action::StopF2BWatcher => {
            if let Some(handle) = self.f2bw_handle.take()  {
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


            }

          }
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
        tui.stop()?;
        break;
      }
    }
    tui.exit()?;
    Ok(())
  }
}
