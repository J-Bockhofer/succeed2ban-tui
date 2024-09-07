use std::fmt;

use crate::{database::schema::{city::City, country::Country, ip::IP, isp::ISP, message::MiniMessage, region::Region}, tasks, themes::Themes};
use rusqlite::{Connection, Result};



use serde::{
  de::{self, Deserializer, Visitor},
  Deserialize, Serialize,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum InputAction { // take stuff from main Action enum?
  ToggleMap,
  ToggleStats,
  ToggleHelp,
  ToggleQuery,
  ToggleBan,
  ToggleLogs,
  SetIOCapacity,

} 

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum StatAction { // take stuff from main Action enum?
  ExitStats,
  Block,
  Unblock,
  Refresh,
  SortAlphabetical,
  SortWarnings,
  SortBlocked,

  NextTimeframe,
  PreviousTimeframe,
} 
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub enum HomeAction {

  DrawAll,
  DrawSticky,
  DrawDecay,

  Query,
  EnterStats,

  Logs,
  Map,
  Clear,

  Ban,

  Unban,

  LogsFirst,
  LogsPrevious,
  LogsNext,
  LogsLast,
  LogsUnselect,
  SetCapacity,
  SubmittedCapacity,

  Follow,
  Static,

}


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

  Stats(StatAction),
  Home(HomeAction),

  EnterNormal,
  EnterTakeAction,
  EnterProcessing,
  ExitProcessing,

  Blank,

  // General Actions
  ConfirmClearLists,
  AbortClearLists,
  ConfirmedClearLists,
  ClearLists,

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
  
  // Query actions
  EnterQuery,
  ExitQuery,
  SubmitQuery(String),
  InvalidQuery,
  QueryNotFound(String),

  // Core
  IONotify(tasks::IOMessage), // String tasks::IOMessage
  //FetchGeo(gen_structs::Geodata),

  // second string is the line, bool is if it came from IO or DB
  GotGeo(IP, tasks::IOMessage, bool),
  //
  /// 0: IP, 1: Line, 2: true if from DB, false if fresh
  PassGeo(IP, tasks::IOMessage, bool),

  InternalLog(String),
  // Ban Actions
  EnterBan,
  ExitBan,
  RequestBan,
  BanIP(IP),
  Banned(bool),

  EnterUnban,
  ExitUnban,
  RequestUnban,
  UnbanIP(IP),
  Unbanned(bool),

  Block(IP),
  //

  StartF2BWatcher,
  StopF2BWatcher,
  StoppedF2BWatcher,
  StartJCtlWatcher,
  StopJCtlWatcher,
  StoppedJCtlWatcher,

  // Startup
  StartupConnect,
  StartupConnected,
  StartupConnectedDB,
  StartupCreateDB,
  StartupDone,
  StartupGotHome(IP),
  // Select Theme, by themename 
  SelectTheme(String),

  // Stats
  StatsShow,
  StatsHide,

  StatsGetCountries,
  StatsGetISPs,
  StatsGetRegions,
  StatsGetCities,

  StatsGotCountry(Country, Vec<MiniMessage>),
  StatsGotISP(ISP, Vec<MiniMessage>),
  StatsGotRegion(Region, Vec<MiniMessage>),
  StatsGotCity(City, Vec<MiniMessage>),

  StatsBlockCountry(Country),
  StatsBlockRegion(Region),
  StatsBlockCity(City),
  StatsBlockISP(ISP),

  StatsUnblockCountry(Country),
  StatsUnblockRegion(Region),
  StatsUnblockCity(City),
  StatsUnblockISP(ISP),

  StatsGetIP(String),
  StatsGotIP(IP),

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
          // Home Actions //
          // General

          "Query" => Ok(Action::Home(HomeAction::Query)),
          "DrawAll" =>  Ok(Action::Home(HomeAction::DrawAll)),
          "DrawSticky" =>  Ok(Action::Home(HomeAction::DrawSticky)),
          "DrawDecay" =>  Ok(Action::Home(HomeAction::DrawDecay)),
          "Stats" => Ok(Action::Home(HomeAction::EnterStats)),
          "Logs" => Ok(Action::Home(HomeAction::Logs)),
          "Map" => Ok(Action::Home(HomeAction::Map)),
          "Clear" => Ok(Action::Home(HomeAction::Clear)),
          "Ban" => Ok(Action::Home(HomeAction::Ban)),
          "Unban" => Ok(Action::Home(HomeAction::Unban)),
          // Logs List / IO Streamed
          "First" => Ok(Action::Home(HomeAction::LogsFirst)),
          "Previous" => Ok(Action::Home(HomeAction::LogsPrevious)),
          "Next" => Ok(Action::Home(HomeAction::LogsNext)),
          "Last" => Ok(Action::Home(HomeAction::LogsLast)),
          "Unselect" => Ok(Action::Home(HomeAction::LogsUnselect)),
          "SetCapacity" => Ok(Action::Home(HomeAction::SetCapacity)),

          "Follow" => Ok(Action::Home(HomeAction::Follow)),
          "Static" => Ok(Action::Home(HomeAction::Static)),


          // Stat Actions //
          // General
          "Home" => Ok(Action::Stats(StatAction::ExitStats)),
          "Block" => Ok(Action::Stats(StatAction::Block)),
          "Unblock" => Ok(Action::Stats(StatAction::Unblock)),          

          // Sorting
          "SortAlphabetical" => Ok(Action::Stats(StatAction::SortAlphabetical)),
          "SortWarnings" => Ok(Action::Stats(StatAction::SortWarnings)),
          "SortBlocked" => Ok(Action::Stats(StatAction::SortBlocked)),

          "NextTimeframe" => Ok(Action::Stats(StatAction::NextTimeframe)),
          "PreviousTimeframe" => Ok(Action::Stats(StatAction::PreviousTimeframe)),
          
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
          _ => Err(E::custom(format!("Unknown Action variant: {}", value))),
        }
      }
    }

    deserializer.deserialize_str(ActionVisitor)
  }
}
