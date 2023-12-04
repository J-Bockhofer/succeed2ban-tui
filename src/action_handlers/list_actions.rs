// Defines Action handling upon receiving input from lists, ie. select next etc.
// In order to not speed up 

use crate::action::Action;

use tokio::{sync::mpsc::UnboundedSender, time::Duration, time};


pub fn schedule_generic_action(tx: UnboundedSender<Action>, action: Action) {
  tokio::spawn(async move {
    tx.send(Action::EnterProcessing).unwrap();
    time::sleep(Duration::from_millis(50)).await;
    tx.send(action).unwrap();
    tx.send(Action::ExitProcessing).unwrap();
  });   
}
