use chrono::Utc;
use rusqlite::Connection;

use crate::{action::Action, geofetcher::{self, deserialize_geolocation, fetch_geolocation}, migrations::schema::ip};

use super::{Mode, Startup};

use color_eyre::eyre::Result;

impl <'a> Startup <'a> {
  pub fn connect(&mut self) -> Result<Option<Action>> {
    let dt = Utc::now();
    let tx = self.action_tx.clone().unwrap();
    fetch_home_and_report(tx);
    self.log_messages.push(format!("{}            Connecting to db", dt.to_string()));
    let conn = Connection::open("iplogs.db")?;
    self.dbconn = Some(conn);
    self.create_db();
    self.get_initial_stats();
    self.mode = Mode::Done;
    Ok(None)
  }
/* 
  pub fn io_notify(&mut self, x: &String) -> Result<Option<Action>> {
        // got new line
        let x = x.clone().deref().to_string();
        let re = self.apptheme.ipregex.clone();

        let results: Vec<&str> = re
          .captures_iter(&x)
          .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
          .collect();

        if results.is_empty() {
          // results were empty, might happen if journalctl sends error message -> in that case just insert the last ip into the message and try again
          if !self.last_ip.is_empty() {
            let msg = x.clone();
            let msg = format!("{} for {}", msg, self.last_ip);
            self.action_tx.clone().unwrap().send(Action::IONotify(msg))?;
          }
          return Ok(None)
        }

        let cip: &str;
        // filtered for IP
        cip = results[0];
        // string contained an IPv4
        let mut is_banned = false;
        if x.contains("Ban") {
          is_banned = true;
        }

        if self.last_ip != String::from(cip) {
          std::thread::sleep(std::time::Duration::from_millis(100));
          // check if is banned
          is_banned = self.f2b_check_banned(cip);
        };

        let conn = self.dbconn.as_ref().unwrap();

        let mut maybe_data = ip::select_ip(conn, cip).unwrap_or_default().take().unwrap_or_default();
        self.last_ip = String::from(cip);
      
        if maybe_data == ip::IP::default() {
          // we have to fetch the data
          let sender = self.action_tx.clone().unwrap();
          self.fetching_ips.push(cip.to_string());
          actions::fetch_geolocation_and_report(cip.to_string(), is_banned.clone(), x.clone(), sender);
        }
        else {
          // data is stored
          maybe_data.is_banned = is_banned;
          self.action_tx.clone().unwrap().send(Action::GotGeo(maybe_data, x.clone(), true))?;  // return true, GeoData came from DB
        }
    Ok(None)
  } */
}




pub fn fetch_geolocation_and_report(ip: String, is_banned: bool, original_message: String, tx: tokio::sync::mpsc::UnboundedSender<Action>) {
    let handle = tokio::task::spawn(async move {      
        let geodat = fetch_geolocation(ip.as_str()).await.unwrap_or(serde_json::Value::default());
        let geodata = deserialize_geolocation(geodat, is_banned);
        if geodata.is_some() {
          tx.send(Action::GotGeo(geodata.unwrap(), original_message, false)).unwrap_or_default(); // false, GeoData was acquired freshly
        } else {
          let fetchmsg = format!("  Could not find location for IP {} ", ip);
          tx.send(Action::InternalLog(fetchmsg)).expect("Fetchlog message failed to send");
        }
      }); 
}

fn fetch_home_and_report(tx: tokio::sync::mpsc::UnboundedSender<Action>) {
  tokio::spawn(async move {
    // get my local ip from somewhere
    let my_local_ip = geofetcher::fetch_home().await.unwrap_or_default();
    let geodat = geofetcher::fetch_geolocation(my_local_ip.to_string().as_str()).await.unwrap_or(serde_json::Value::default());
    let geodata = geofetcher::deserialize_geolocation(geodat, false);

    if geodata.is_some() {
      tx.send(Action::StartupGotHome(geodata.unwrap())).unwrap_or_default();          
    } else {
      tx.send(Action::StartupGotHome(ip::IP::default())).unwrap_or_default();
    }
  });
}



