use std::io::{Seek, BufReader, BufRead};
use notify::{Watcher, RecursiveMode, Result, RecommendedWatcher, Config, Event};
use serde::Serialize;
use std::sync::mpsc::Receiver;

use tokio::{
    sync::mpsc::UnboundedSender,
    sync::mpsc::UnboundedReceiver,
  };

use tokio_util::sync::CancellationToken;

use crate::action::Action;

use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;


use std::process::{Command, Stdio, ChildStdout};
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

// pass another receiver for the cancellation token? // _cancellation_token: CancellationToken)
pub async fn notify_change(path: &str, _event_tx:UnboundedSender<Action>, rx: Receiver<Result<Event>>) -> Result<()> {
  let mut pos = std::fs::metadata(path)?.len();
  let mut f = std::fs::File::open(path)?;
/*     // get file
    let mut pos = std::fs::metadata(path)?.len();
    // get pos to end of file
    let mut f = std::fs::File::open(path)?;
    
    // set up watcher
    let (tx, rx) = std::sync::mpsc::channel();
    let mut watcher: notify::INotifyWatcher = RecommendedWatcher::new(tx, Config::default())?;
    watcher.watch(path.as_ref(), RecursiveMode::NonRecursive)?; */
/*     tokio::select! {
      _ = _f2b_cancellation_token.cancelled() => {
        drop(watcher);
        //_res.abort();
        todo!("Got dropped, so all should be fine");
        //return;
      }
      _ = _res => {

        todo!("Notify Change ended before it was cancelled, should not happen")
      }
    } */

    // watch
    for res in rx {

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

                //let addedlines = msgs.join("++++");
                //println!("> {}", addedlines);
                //log::info!("Added potential multiline string");
                //tokio::time::sleep(tokio::time::Duration::from_millis(50)).await;
                //_event_tx.send(Action::IONotify(addedlines)).unwrap();
            }
            Err(error) => { println!("{error:?}")},
        }
    }

    Ok(())
}



pub async fn monitor_journalctl(event_tx:UnboundedSender<Action>, mut cancel_rx:UnboundedReceiver<bool>) -> Result<()> {

    let (action_tx, mut action_rx) = tokio::sync::mpsc::unbounded_channel::<String>();

    let (_cancel_tx, mut _cancel_rx) = tokio::sync::mpsc::unbounded_channel::<bool>();

    let argus = vec!["-n", "1", "-f", "-u", "ssh"];
    let mut command = Command::new("journalctl");

    let _joinhandle = tokio::spawn(async move {
  
      // closure needs a sender to send to new string and a receiver to terminate the process
      let mut tick_interval = tokio::time::interval(std::time::Duration::from_nanos(10));
      command
          .args(argus)
          .stdout(Stdio::piped());
  
      if let Ok(mut child) = command.spawn() {
          if let Some(mut out) = child.stdout.take() {
            let mut pre_line: Vec<u8> = vec![];
              loop {            
                let mut chars: [u8; 1] = [0; 1];
                out.read(&mut chars).expect("didn't work");
                
                // check if we encounter a newline character, if yes send chars 
                if chars[0] == b'\n' || chars[0] == b'\r' {
                  // terminate string
                  //println!("Newline detected");
                  //println!("{}", str::from_utf8(&pre_line).unwrap());
                  let line = String::from_utf8(pre_line).unwrap();
                  action_tx.send(line).unwrap_or_else(|err| {
                    println!("Send Error: {}", err);
                  });
                  pre_line = vec![];
                }
                else {
                  // add char to line
                  //print!("Pushing {}", str::from_utf8(&chars).unwrap());
                  pre_line.push(chars[0]);
                  //println!("{}", str::from_utf8(&pre_line).unwrap());
                } 
                let cancel = _cancel_rx.try_recv().unwrap_or_default();
                if cancel {
                    let _ = child.kill().unwrap();
                    break;
                }  
                tick_interval.tick().await;   
              }  
          } else {

          }
      } else {
        println!("Process failed to start");
      }

  
    });
  
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_millis(100));
    loop {
       let msg = action_rx.try_recv().unwrap_or_default();
       if !msg.is_empty() {
        event_tx.send(Action::IONotify(IOMessage::SingleLine(msg, IOProducer::Journal))).unwrap();
        //println!("Received {}", msg);
       }
       

       let cancel = cancel_rx.try_recv().unwrap_or_default();
       if cancel {
        _cancel_tx.send(true).unwrap();
        tick_interval.tick().await;
        _cancel_tx.send(true).unwrap();

        event_tx.send(Action::StoppedJCtlWatcher).unwrap();
        _joinhandle.abort();

       }       


       tick_interval.tick().await;
    }

}



pub async fn monitor_journalctl_2(event_tx:UnboundedSender<Action>, _cancellation_token: CancellationToken) -> Result<()> {


  let argus = vec!["-n", "1", "-f", "-u", "ssh"];
  let mut command = Command::new("journalctl");

  tokio::spawn(async move {

    // closure needs a sender to send to new string and a receiver to terminate the process
    let mut tick_interval = tokio::time::interval(std::time::Duration::from_nanos(10));
    command
        .args(argus)
        .stdout(Stdio::piped());

    if let Ok(mut child) = command.spawn() {
        if let Some(mut out) = child.stdout.take() {
          let mut pre_line: Vec<u8> = vec![];
            loop {            
              let mut chars: [u8; 1] = [0; 1];
              out.read(&mut chars).expect("reading journal buf didn't work");
              
              // check if we encounter a newline character, if yes send chars 
              if chars[0] == b'\n' || chars[0] == b'\r' {
                // terminate string
                //println!("Newline detected");
                //println!("{}", str::from_utf8(&pre_line).unwrap());
                let line = String::from_utf8(pre_line).unwrap();
                event_tx.send(Action::IONotify(IOMessage::SingleLine(line, IOProducer::Journal))).unwrap_or_else(|err| {
                  println!("Send Error: {}", err);
                });
                pre_line = vec![];
              }
              else {
                // add char to line
                //print!("Pushing {}", str::from_utf8(&chars).unwrap());
                pre_line.push(chars[0]);
                //println!("{}", str::from_utf8(&pre_line).unwrap());
              } 
              let cancel = _cancellation_token.is_cancelled();
              if cancel {
                  let fetchmsg = format!(" ❌ STOPPED journalctl watcher process");
                  event_tx.send(Action::InternalLog(fetchmsg)).expect("LOG: StopJCTLWatcher message failed to send");
                  let _ = child.kill().unwrap();
                  break;
              }  
              tick_interval.tick().await;   
            }  
        } else {

        }
    } else {
      println!("Process failed to start");
    }
  });

  Ok(())

}