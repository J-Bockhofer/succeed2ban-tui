use crate::{action::Action, tasks};

use super::App;

use tokio_util::sync::CancellationToken;
use color_eyre::eyre::Result;

use notify::{Event, INotifyWatcher, RecommendedWatcher, Watcher};
use tokio::sync::mpsc::UnboundedSender;
use tokio::time::{sleep, Duration};


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

        let _resp = tasks::monitor_journalctl( action_tx2, _jctl_cancellation_token).await?;

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
