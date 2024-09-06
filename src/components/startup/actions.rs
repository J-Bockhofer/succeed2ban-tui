use chrono::Utc;
use rusqlite::Connection;

use crate::{action::Action, geofetcher::{self, deserialize_geolocation, fetch_geolocation}, database::schema::{city, country, ip::{self, IP}, isp, message, region}, tasks::{self, IOMessage, IOProducer}};

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

  pub fn io_notify(&mut self, iomsg: IOMessage) -> Result<Option<Action>> {
       // got new line
       let (catmsg, prod) = iomsg.destructure(" ");
        // regex for finding an IPv4
        let results: Vec<&str> = self.apptheme.ipregex
          .captures_iter(&catmsg)
          .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
          .collect();
        if results.is_empty() {
          // results were empty, might happen if journalctl sends error message -> in that case just insert the last ip into the message and try again
          if !self.last_ip.is_empty() && prod == IOProducer::Journal {
            let msg = crate::tasks::IOMessage::SingleLine(format!("{} for {}", catmsg, self.last_ip), prod);
            self.action_tx.clone().unwrap().send(Action::IONotify(msg))?;
          }
          return Ok(None)
        }
        // Just take the first occurance of an ip into account
        let cip = results[0];
        let mut is_banned = catmsg.contains("Ban");
        if self.last_ip != String::from(cip) {
          std::thread::sleep(std::time::Duration::from_millis(50));
          // check if is banned
          is_banned = self.f2b_check_banned(cip);
        };
        self.last_ip = String::from(cip);

        let conn = self.dbconn.as_ref().unwrap();
        let mut maybe_data = ip::select_ip(conn, cip).unwrap_or_default().take().unwrap_or_default();
        
        if maybe_data == ip::IP::default() {
          // we have to fetch the data
          let sender = self.action_tx.clone().unwrap();
          self.fetching_ips.push(cip.to_string());
          fetch_geolocation_and_report(cip.to_string(), is_banned.clone(), iomsg, sender);
        }
        else {
          // data is stored
          maybe_data.is_banned = is_banned;
          self.action_tx.clone().unwrap().send(Action::GotGeo(maybe_data, iomsg, true))?;  // return true, GeoData came from DB
        }
    Ok(None)
  }

  pub fn got_geo(&mut self, x: IP, iomsg: IOMessage, from_db: bool) -> Result<Option<Action>> {
      
      let conn = self.dbconn.as_ref().unwrap();

      let meta = crate::database::schema::update_db_on_new_log(conn, x.clone(), from_db);
      crate::database::schema::update_ip_db_on_new_log(conn, x.clone(), from_db);

      let (catmsg, prod) = iomsg.destructure(" ");

      let is_jctl: bool = prod == IOProducer::Journal;
      let is_ban  = catmsg.contains("Ban");
      let tx = self.action_tx.clone().unwrap();
      tx.send(Action::PassGeo(x.clone(), iomsg.clone(), from_db)).expect("PassGeo failed to send");
      let symb = if from_db {self.apptheme.symbol_db.clone()} else {self.apptheme.symbol_reqwest.clone()};
      let fetchmsg = format!(" {} Got location for IP {} ", symb, x.ip);
      tx.send(Action::InternalLog(fetchmsg)).expect("Fetchlog message failed to send");

      if meta.country.is_blocked || meta.city.is_blocked || meta.isp.is_blocked || meta.region.is_blocked { 
        if !x.is_banned && !is_ban {
          tx.send(Action::BanIP(x.clone())).expect("Block failed to send");
          let timestamp = chrono::offset::Local::now().to_rfc3339();
          let mut reasons: Vec<String> = vec![];
          if meta.country.is_blocked {reasons.push(format!("Country: {}", meta.country.name));}
          if meta.region.is_blocked {reasons.push(format!("Region: {}", meta.region.name));}
          if meta.city.is_blocked {reasons.push(format!("City: {}", meta.city.name));}
          if meta.isp.is_blocked {reasons.push(format!("ISP: {}", meta.isp.name));}

          let blockmsg = format!(" {} Blocked IP {} ", self.apptheme.symbol_block, x.ip);
          tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");
          for reason in reasons {
            let blockmsg = format!(" {} Blocked {} ",self.apptheme.symbol_block , reason);
            tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");
          }
        } else {
          let blockmsg = format!(" {} IP already blocked {} ", self.apptheme.symbol_block, x.ip);
          tx.send(Action::InternalLog(blockmsg)).expect("Blocklog message failed to send");            
        }       

      }

      let timestamp = chrono::offset::Local::now().to_rfc3339();
      match iomsg {
        IOMessage::SingleLine(msg, _) => {
          message::insert_new_message(conn, Option::None, &timestamp, &msg, &x.ip, &x.country, &x.region, &x.city, &x.isp, is_jctl, is_ban).unwrap();
        },
        IOMessage::MultiLine(vx, _) => {
          for msg in vx {
            message::insert_new_message(conn, Option::None, &timestamp, &msg, &x.ip, &x.country, &x.region, &x.city, &x.isp, is_jctl, is_ban).unwrap();
          }
        },
      };
      Ok(None)
  }

}

fn fetch_geolocation_and_report(ip: String, is_banned: bool, original_message: tasks::IOMessage, tx: tokio::sync::mpsc::UnboundedSender<Action>) {
    tokio::task::spawn(async move {      
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



