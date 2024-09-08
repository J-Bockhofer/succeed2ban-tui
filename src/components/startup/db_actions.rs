use rusqlite::Connection;

use crate::{action::Action, database::schema::{ip::{self, IP}, message}, app::models::{IOMessage, IOProducer}};


pub fn process_query(conn: &Connection, querystr: String, tx: tokio::sync::mpsc::UnboundedSender<Action>) {
    let ip = ip::select_ip(conn, querystr.as_str()).unwrap_or_default().unwrap_or_default();
    let opmsgs = message::select_message_by_ip(conn, querystr.as_str()).unwrap();
    let mut actmsgs: Vec<message::Message> = vec![];

    for opmsg in opmsgs {
      let msg = opmsg.unwrap_or(message::Message::default());
      if msg != message::Message::default() {actmsgs.push(msg);}
    }

    if ip == ip::IP::default() {
      // send back query not found
      tx.send(Action::QueryNotFound(querystr)).expect("QueryNotFound failed to send!");
    } else {
      // spawn thread to send debounced messages
      tokio::spawn(async move{
        for msg in actmsgs {
          let prod = if msg.is_jctl {IOProducer::Journal} else {IOProducer::Log};
          tx.send(Action::PassGeo(ip.clone(), IOMessage::SingleLine(msg.text, prod), true)).expect("PassGeo failed to send on query!"); 
          tokio::time::sleep(tokio::time::Duration::from_millis(5)).await;}
          // inefficient but else but require me to set up a duplicate receiver or refactor receive function
      });
    }        
}