use crate::action::Action;

use super::App;

use tokio_util::sync::CancellationToken;
use color_eyre::eyre::Result;

use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, RecommendedWatcher, Config, Event};
use serde::Serialize;
use tokio::time::{interval, Duration};
use tokio::{
    sync::mpsc::UnboundedSender,
    sync::mpsc::UnboundedReceiver,
    sync::mpsc::Receiver,
  };

use super::models::{IOMessage, IOProducer};

impl App {

    pub async fn start_f2b_watcher(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        // cancel any existing watcher
        if !self.f2b_cancellation_token.is_cancelled() {
          self.f2b_cancellation_token.cancel();
          tokio::time::sleep(tokio::time::Duration::from_millis(100)).await; // make sure we're wound down
        }
    
        // get the log path
        let path = if self.config.logpath.is_empty() {self.f2b_logpath.clone()} else {self.config.logpath.clone()};
        log::info!("{}", path);
    
        // clone sender for moving into the watcher
        let action_tx2 = action_tx.clone();
    
        // create new cancellation token
        let token = CancellationToken::new();    
        // create new child token for watcher process          
        let _f2b_cancellation_token = token.child_token();
        self.f2b_cancellation_token = token;
        
        // construct the watcher
        let (atx, arx) = tokio::sync::mpsc::channel(100);
        let mut watcher: notify::INotifyWatcher = notify::RecommendedWatcher::new(move |result: std::result::Result<Event, notify::Error>| {
          atx.blocking_send(result).expect("Failed to send event");
        }, notify::Config::default())?;
        watcher.watch(path.as_ref(), notify::RecursiveMode::NonRecursive)?;
    
        // construct the listener
        let filewatcher = tokio::spawn(async move  {
            log::info!("Started f2b watcher");
            let _ = monitor_ionotify_file(&path, action_tx2, arx, _f2b_cancellation_token).await;
            log::info!("Dropping f2b watcher");
            drop(watcher);
          });
    
        // log success
        let fetchmsg = format!(" ✔ STARTED fail2ban watcher");
        action_tx.send(Action::InternalLog(fetchmsg)).expect("LOG: StartF2BWatcher message failed to send");
        Ok(())
      }
    
    pub async fn stop_f2b_watcher(&mut self, action_tx: UnboundedSender<Action>) -> Result<()> {
        // try to cancel and wait
        self.f2b_cancellation_token.cancel();
        tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
        if self.f2b_cancellation_token.is_cancelled() {
            log::info!("Stopped f2b watcher");
            action_tx.send(Action::StoppedF2BWatcher).expect("LOG: StoppedF2BWatcher message failed to send");
        } else {
            tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
            action_tx.send(Action::StopF2BWatcher).unwrap();
        }
        let fetchmsg = format!(" ❌ STOPPED fail2ban watcher");
        action_tx.send(Action::InternalLog(fetchmsg)).expect("LOG: StopF2BWatcher message failed to send");
        Ok(())
    }    

}

use notify::Result as NResult;

pub async fn monitor_ionotify_file(path: &str, _event_tx:UnboundedSender<Action>, mut rx: Receiver<NResult<Event>>, _cancellation_token: CancellationToken) -> NResult<()> {
  let mut pos = std::fs::metadata(path)?.len();
  let mut f = std::fs::File::open(path)?;

  loop {
    tokio::select! {
      _ = _cancellation_token.cancelled() => {
        return Ok(())
      }
      _res = rx.recv() => {
        if let Some(res) = _res {
          match res {
            Ok(_event) => {
                // ignore any event that didn't change the pos
                if f.metadata()?.len() == pos {
                    continue;
                }
                if f.metadata()?.len() == 0 {
                    continue;
                }

                // read from pos to end of file
                f.seek(std::io::SeekFrom::Start(pos))?;

                // update post to end of file
                pos = f.metadata()?.len();
                let reader = BufReader::new(&f);
                let mut msgs = vec!["".to_string()];

                for line in reader.lines() {
                    let cline = line.unwrap();
                    let nullchar = cline.chars().nth(cline.chars().count());
                    if !nullchar.is_none()
                    {
                        let newchar = nullchar.unwrap();
                        if newchar.is_whitespace()
                        {
                            pos -= 1;
                        }

                    }
                    if !cline.is_empty() {
                      msgs.push(cline.clone());                 
                    }
                }
                
                let msg = if msgs.len() == 1 {
                  IOMessage::SingleLine(msgs[0].clone(), IOProducer::Log)
                } else {
                  IOMessage::MultiLine(msgs, IOProducer::Log)
                };
                _event_tx.send(Action::IONotify(msg)).unwrap();

            }
            Err(error) => { log::error!("Logwatcher failed with: {error:?}"); return Err(error)},
          }
        }
      }
    }
  }
}
