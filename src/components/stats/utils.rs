use chrono::{DateTime, Datelike, FixedOffset};

use super::StatIP;
use crate::database::schema::message::MiniMessage;

pub fn convert_strings_to_utc(strings: Vec<String>) -> Vec<DateTime<FixedOffset>> {
  let mut ts: Vec<DateTime<FixedOffset>> = vec![];
  for stampstr in strings {
    let cts: DateTime<FixedOffset> = DateTime::parse_from_rfc3339(&stampstr).unwrap();
    ts.push(cts);
  }
  ts.sort_by(|a, b| a.cmp(b));
  ts
}

pub fn get_msgs_per_ip(msgs: Vec<MiniMessage>) -> Vec<StatIP> {
  let mut ipvec: Vec<StatIP> = vec![];
  if msgs.is_empty() {
    return ipvec;
  }
  let mut last_ip = msgs[0].ip.clone();
  let mut last_timestamps: Vec<String> = vec![];
  let mut num_this: usize = 0;
  for msg in msgs {
    let cip = msg.ip;
    if cip == last_ip {
      last_timestamps.push(msg.created_at);
      num_this = num_this.saturating_add(1);
    } else {
      let rt: Vec<DateTime<FixedOffset>> = convert_strings_to_utc(last_timestamps);
      let statip = StatIP { ip: last_ip, timestamps: rt, warnings: num_this };
      ipvec.push(statip);
      last_ip = cip;
      num_this = 0;
      last_timestamps = vec![];
    }
  }

  if ipvec.is_empty() {
    let rt: Vec<DateTime<FixedOffset>> = convert_strings_to_utc(last_timestamps);
    let statip = StatIP { ip: last_ip, timestamps: rt, warnings: num_this };
    ipvec.push(statip);
  }

  ipvec
}

