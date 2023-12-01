// Defines Action handling upon receiving input from lists, ie. select next etc.
// In order to not speed up 

use crate::action::Action;

use tokio::{sync::mpsc::UnboundedSender, time::Duration, time};



pub fn schedule_next_loglist(tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
      tx.send(Action::EnterProcessing).unwrap();
      time::sleep(Duration::from_millis(100)).await;
      tx.send(Action::LogsNext).unwrap();
      tx.send(Action::ExitProcessing).unwrap();
    });    
}

pub fn schedule_previous_loglist(tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
      tx.send(Action::EnterProcessing).unwrap();
      time::sleep(Duration::from_millis(100)).await;
      tx.send(Action::LogsPrevious).unwrap();
      tx.send(Action::ExitProcessing).unwrap();
    });    
}

pub fn schedule_first_loglist(tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
      tx.send(Action::EnterProcessing).unwrap();
      time::sleep(Duration::from_millis(100)).await;
      tx.send(Action::LogsFirst).unwrap();
      tx.send(Action::ExitProcessing).unwrap();
    });    
}

pub fn schedule_last_loglist(tx: UnboundedSender<Action>) {
    tokio::spawn(async move {
      tx.send(Action::EnterProcessing).unwrap();
      time::sleep(Duration::from_millis(100)).await;
      tx.send(Action::LogsLast).unwrap();
      tx.send(Action::ExitProcessing).unwrap();
    });    
}