use crate::action::Action;

use super::App;

use tokio::io::AsyncSeekExt;
use tokio::io::AsyncReadExt;
use tokio::process::Command;
use tokio_util::sync::CancellationToken;
use color_eyre::eyre::Result;

use notify::{Event, INotifyWatcher, RecommendedWatcher, Watcher};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration, interval};

use std::process::{Stdio, ChildStdout};
use std::io::Read;

use super::models::{IOMessage, IOProducer};

impl App {

    pub async fn start_jctl_watcher(&mut self, action_tx: &UnboundedSender<Action>) -> Result<()> {

        self.jctl_cancellation_token.cancel();
        sleep(Duration::from_millis(50)).await; // make sure we're wound down

        // create cancellation token
        let token = CancellationToken::new();
        let _jctl_cancellation_token = token.child_token();
        self.jctl_cancellation_token = token;

        // start the fail2ban watcher
        let action_tx2 = action_tx.clone();

        let _resp = monitor_journalctl( action_tx2, _jctl_cancellation_token).await?;

        //self.jctl_handle = Option::Some(journalwatcher);
        let fetchmsg = format!(" ✔ STARTED journalctl watcher");
        action_tx.send(Action::InternalLog(fetchmsg)).expect("LOG: StartJCTLWatcher message failed to send");

        Ok(())
    }
        
    pub async fn stop_jctl_watcher(&mut self, action_tx: &UnboundedSender<Action>) -> Result<()> {

        self.jctl_cancellation_token.cancel();
        sleep(Duration::from_millis(50)).await;

        if self.jctl_cancellation_token.is_cancelled() {
          let fetchmsg = format!(" ❌ STOPPED journalctl watcher");
          action_tx.send(Action::InternalLog(fetchmsg)).expect("LOG: StopJCTLWatcher message failed to send");
          log::info!("Stopped jctl watcher");
          action_tx.send(Action::StoppedJCtlWatcher).expect("LOG: StoppedF2BWatcher message failed to send");
        } else {
          sleep(Duration::from_millis(10)).await;
          action_tx.send(Action::StopJCtlWatcher).unwrap();
        }

        Ok(())
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