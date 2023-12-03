use std::fmt;

use crate::migrations::schema;
use rusqlite::{Connection, Result};



use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum Action {
  Tick,
  Render,
  Resize(u16, u16),
  Suspend,
  Resume,
  Quit,
  Refresh,
  Error(String),
  Help,

  EnterNormal,
  EnterTakeAction,
  EnterProcessing,
  ExitProcessing,

  Blank,

  // List state actions
  // -- LOG LIST -- iostreamed
  LogsScheduleNext,
  LogsSchedulePrevious,
  LogsNext,
  LogsPrevious,
  LogsScheduleFirst,
  LogsScheduleLast,
  LogsFirst,
  LogsLast,
  // -- IP LIST -- iplist
  IPsScheduleNext,
  IPsSchedulePrevious,
  IPsNext,
  IPsPrevious,
  IPsUnselect,
  //IPs
  // -- ACTION LIST -- available_actions
  ActionsScheduleNext,
  ActionsSchedulePrevious,
  ActionsNext,
  ActionsPrevious,
  ActionsExecute,
  


  IONotify(String),
  //FetchGeo(gen_structs::Geodata),

  GotGeo(schema::IP, String),
  Ban,
  BanIP(String),
  Banned(bool),
  StartF2BWatcher,
  StopF2BWatcher,
  StartJCtlWatcher,
  StopJCtlWatcher,
  StoppedJCtlWatcher,

  StartupConnect,
  StartupConnected,


  StartupConnectedDB,
  StartupCreateDB,


  StartupDone,
}

impl<'de> Deserialize<'de> for Action {
  fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
  where
    D: Deserializer<'de>,
  {
    struct ActionVisitor;

    impl<'de> Visitor<'de> for ActionVisitor {
      type Value = Action;

      fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a valid string representation of Action")
      }

      fn visit_str<E>(self, value: &str) -> Result<Action, E>
      where
        E: de::Error,
      {
        match value {
          "Tick" => Ok(Action::Tick),
          "Render" => Ok(Action::Render),
          "Suspend" => Ok(Action::Suspend),
          "Resume" => Ok(Action::Resume),
          "Quit" => Ok(Action::Quit),
          "Refresh" => Ok(Action::Refresh),
          "Help" => Ok(Action::Help),
          "EnterNormal" => Ok(Action::EnterNormal),
          "EnterTakeAction" => Ok(Action::EnterTakeAction),
          "StartupDone" => Ok(Action::StartupDone),
          // Error
          data if data.starts_with("Error(") => {
            let error_msg = data.trim_start_matches("Error(").trim_end_matches(")");
            Ok(Action::Error(error_msg.to_string()))
          },
          // Resize
          data if data.starts_with("Resize(") => {
            let parts: Vec<&str> = data.trim_start_matches("Resize(").trim_end_matches(")").split(',').collect();
            if parts.len() == 2 {
              let width: u16 = parts[0].trim().parse().map_err(E::custom)?;
              let height: u16 = parts[1].trim().parse().map_err(E::custom)?;
              Ok(Action::Resize(width, height))
            } else {
              Err(E::custom(format!("Invalid Resize format: {}", value)))
            }
          },
          // IONotify
          data if data.starts_with("IONotify(") => {
            let notify_msg = data.trim_start_matches("IONotify(").trim_end_matches(")");
            Ok(Action::IONotify(notify_msg.to_string()))            
          },
          _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
        }
      }
    }

    deserializer.deserialize_str(ActionVisitor)
  }
}
