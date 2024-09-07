use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config, Event};
use serde::Serialize;
use tokio::time::{interval, Duration};
use tokio::{
    sync::mpsc::UnboundedSender,
    sync::mpsc::UnboundedReceiver,
    sync::mpsc::Receiver,
  };

use tokio_util::sync::CancellationToken;

use crate::action::Action;

use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;
use tokio::process::Command;

use std::process::{Stdio, ChildStdout};
use std::io::Read;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IOProducer {
  Journal,
  Log
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum IOMessage {
  SingleLine(String, IOProducer),
  MultiLine(Vec<String>, IOProducer),
}

impl IOMessage {
  pub fn destructure(&self, sep: &str) -> (String, IOProducer) {
    let prod: IOProducer;
    let catmsg: String;
    match self.clone() {
      IOMessage::SingleLine(x, p) => {
        prod = p;
        catmsg = x},
      IOMessage::MultiLine(vx, p) => {
        prod = p;
        catmsg = vx.join(sep)
      },
    }
    return (catmsg, prod)
  }
}

pub async fn monitor_ionotify_file(path: &str, _event_tx:UnboundedSender<Action>, mut rx: Receiver<Result<Event>>, _cancellation_token: CancellationToken) -> Result<()> {
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

/// Creates a sender and receiver process for reading journalctl's stdout. This function justs sets them up and returns immediately.
pub async fn monitor_journalctl(action_tx:UnboundedSender<Action>, cancel_token: CancellationToken) -> Result<()> {


  let (stdout_tx, mut stdout_rx) = tokio::sync::mpsc::unbounded_channel::<String>();
  //let mut command = Command::new("tail");
  //let argus = vec!["-n", "1", "-f", "~/Dev/RUST/journal.txt"];
  
  let argus = vec!["-n", "1", "-f", "-u", "ssh"];
  let mut command = Command::new("journalctl");
  command.args(argus).stdout(Stdio::piped());

  let child_token = cancel_token.child_token();
  
  //sender
  let _joinhandle = tokio::spawn(async move {
    log::info!("Start jctl sender");
    let mut tick_interval = interval(Duration::from_nanos(10));
    if let Ok(mut child) = command.spawn() {
        if let Some(mut out) = child.stdout.take() {
          
          let mut pre_line: Vec<u8> = vec![];
          let mut buf: [u8; 1] = [0; 1];
            loop {            
              

              tokio::select! {
                _ = child_token.cancelled() => {
                  log::info!("Stopped jctl sender");
                  break;
                },
                read_result = out.read(&mut buf) => {

                  match read_result {
                    Ok(0) => break, // End of file reached or no more output
                    Ok(_) => {
                        // Check if we encounter a newline character, if yes send chars
                        if buf[0] == b'\n' || buf[0] == b'\r' {
                            let line = String::from_utf8(pre_line.clone()).unwrap();
                            stdout_tx.send(line).unwrap_or_else(|err| {
                                log::error!("JournalCtl Error: {}", err);
                            });
                            pre_line.clear();
                        } else {
                            pre_line.push(buf[0]);
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading stdout: {:?}", e);
                        break;
                    }
                  }

                }
              }
              tick_interval.tick().await;
            }  
        } else {

        }
    } else {
      log::error!("Process failed to start");
    }
  });

  // receiver
  tokio::spawn(async move {
    log::info!("Start jctl receiver");
    loop {
        tokio::select! {
          _ = cancel_token.cancelled() => {
            log::info!("Stopped jctl receiver");
            break;
          }
          maybe_msg = stdout_rx.recv() => {
            if let Some(msg) = maybe_msg {
              if !msg.is_empty() {
                action_tx.send(Action::IONotify(IOMessage::SingleLine(msg, IOProducer::Journal))).unwrap();
              }
            }
          }
        }
    }
  });

  Ok(())
}