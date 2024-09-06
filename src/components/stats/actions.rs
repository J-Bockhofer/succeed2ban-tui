//! Defines functions for Stats that happen on Key Events
use color_eyre::eyre::Result;
use tokio::sync::mpsc::UnboundedSender;
use crate::{action::Action, database::schema::ip::IP};
use tokio::time::{self, Duration};
use super::{Stats, enums::{SelectionMode, SortMode, SortState}};

///  Fetches Country data from DB, also fetches all related data (City, Region, ISP)
pub fn refresh_countries(tx: UnboundedSender<Action>) -> Result<()> {
  // 🔃
  tokio::spawn(async move {
    tx.send(Action::StatsGetCountries).expect("Failed to refresh countries; E404");
    time::sleep(Duration::from_millis(25)).await;
    tx.send(Action::StatsGetRegions).expect("Failed to refresh regions; E404");
    time::sleep(Duration::from_millis(25)).await;
    tx.send(Action::StatsGetCities).expect("Failed to refresh cities; E404");
    time::sleep(Duration::from_millis(25)).await;
    tx.send(Action::StatsGetISPs).expect("Failed to refresh ISPs; E404");
    time::sleep(Duration::from_millis(5)).await;
    let fetchmsg = format!(" 🔃 Refreshed Stats ");
    tx.send(Action::InternalLog(fetchmsg)).expect("LOG: Refresh stats message failed to send");
  });
  Ok(())
}

// BLOCKING Country// --------------------------------------------------------------- //
pub fn block_selected_country(stats: &mut Stats) -> Result<()> {
  if stats.countries.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.countries.state.selected().unwrap();
  let sel_country = stats.countries.items[sel_idx].clone().0;
  tx.send(Action::StatsBlockCountry(sel_country)).expect("Failed to send request to block Country");
  stats.countries.items[sel_idx].0.is_blocked = true;
  Ok(())
}

pub fn unblock_selected_country(stats: &mut Stats) -> Result<()> {
  if stats.countries.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.countries.state.selected().unwrap();
  let sel_country = stats.countries.items[sel_idx].clone().0;
  tx.send(Action::StatsUnblockCountry(sel_country)).expect("Failed to send request to unblock Country");
  stats.countries.items[sel_idx].0.is_blocked = false;
  Ok(())
}

// BLOCKING Region// --------------------------------------------------------------- //
pub fn block_selected_region(stats: &mut Stats) -> Result<()> {
  if stats.regions.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.regions.state.selected().unwrap();
  let sel_region = stats.regions.items[sel_idx].clone().0;
  tx.send(Action::StatsBlockRegion(sel_region)).expect("Failed to send request to block Region");
  stats.regions.items[sel_idx].0.is_blocked = true;
  Ok(())
}

pub fn unblock_selected_region(stats: &mut Stats) -> Result<()> {
  if stats.regions.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.regions.state.selected().unwrap();
  let sel_region = stats.regions.items[sel_idx].clone().0;
  tx.send(Action::StatsUnblockRegion(sel_region)).expect("Failed to send request to unblock Region");
  stats.regions.items[sel_idx].0.is_blocked = false;
  Ok(())
}

// BLOCKING City// --------------------------------------------------------------- //
pub fn block_selected_city(stats: &mut Stats) -> Result<()> {
  if stats.cities.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.cities.state.selected().unwrap();
  let sel_city = stats.cities.items[sel_idx].clone().0;
  tx.send(Action::StatsBlockCity(sel_city)).expect("Failed to send request to block City");
  stats.cities.items[sel_idx].0.is_blocked = true;
  Ok(())
}

pub fn unblock_selected_city(stats: &mut Stats) -> Result<()> {
  if stats.cities.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.cities.state.selected().unwrap();
  let sel_city = stats.cities.items[sel_idx].clone().0;
  tx.send(Action::StatsUnblockCity(sel_city)).expect("Failed to send request to unblock City");
  stats.cities.items[sel_idx].0.is_blocked = false;
  Ok(())
}

// BLOCKING ISP// --------------------------------------------------------------- //
pub fn block_selected_isp(stats: &mut Stats) -> Result<()> {
  if stats.isps.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.isps.state.selected().unwrap();
  let sel_isp = stats.isps.items[sel_idx].clone().0;
  tx.send(Action::StatsBlockISP(sel_isp)).expect("Failed to send request to block ISP");
  stats.isps.items[sel_idx].0.is_blocked = true;
  Ok(())
}

pub fn unblock_selected_isp(stats: &mut Stats) -> Result<()> {
  if stats.isps.items.is_empty() {return Ok(())}
  let tx = stats.action_tx.clone().unwrap();
  let sel_idx = stats.isps.state.selected().unwrap();
  let sel_isp = stats.isps.items[sel_idx].clone().0;
  tx.send(Action::StatsUnblockISP(sel_isp)).expect("Failed to send request to unblock ISP");
  stats.isps.items[sel_idx].0.is_blocked = false;
  Ok(())
}

// BLOCKING IP// --------------------------------------------------------------- //
pub fn block_selected_ip(stats: &mut Stats) -> Result<()> {
  if stats.ips.items.is_empty() {return Ok(())}
  if stats.selected_ip == IP::default() {return Ok(())}

  let tx = stats.action_tx.clone().unwrap();
  let sel_ip = stats.selected_ip.clone();
  if sel_ip.is_banned {return Ok(())}

  tx.send(Action::BanIP(sel_ip)).expect("Failed to send request to block IP");
  stats.selected_ip.is_banned = true;
  Ok(())
}

pub fn unblock_selected_ip(stats: &mut Stats) -> Result<()> {
  if stats.ips.items.is_empty() {return Ok(())}
  if stats.selected_ip == IP::default() {return Ok(())}

  let tx = stats.action_tx.clone().unwrap();
  let sel_ip = stats.selected_ip.clone();
  if !sel_ip.is_banned {return Ok(())}

  tx.send(Action::UnbanIP(sel_ip)).expect("Failed to send request to block IP");
  stats.selected_ip.is_banned = true;
  Ok(())
}

// SORT ALPHANUM// --------------------------------------------------------------- //
pub fn sort_by_alphabetical(stats: &mut Stats) -> Result<()> {
  let _ =  match stats.selection_mode {
    SelectionMode::Country => {
      match stats.countries_sort {
        SortState::AlphabeticalRev => {
          stats.countries.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.countries.items.reverse();
          stats.countries_sort = SortState::Alphabetical;},
        SortState::Alphabetical => {
          stats.countries.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.countries_sort = SortState::AlphabeticalRev;
        },
        _ => {stats.countries_sort = SortState::Alphabetical;},
      }
    },
    SelectionMode::Region => {
      match stats.regions_sort {
        SortState::AlphabeticalRev => {
          stats.regions.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.regions.items.reverse();
          stats.regions_sort = SortState::Alphabetical;},
        SortState::Alphabetical => {
          stats.regions.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.regions_sort = SortState::AlphabeticalRev;
        },
        _ => {stats.regions_sort = SortState::Alphabetical;},
      }
    },
    SelectionMode::City => {
      match stats.cities_sort {
        SortState::AlphabeticalRev => {
          stats.cities.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.cities.items.reverse();
          stats.cities_sort = SortState::Alphabetical;},
        SortState::Alphabetical => {
          stats.cities.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.cities_sort = SortState::AlphabeticalRev;
        },
        _ => {stats.cities_sort = SortState::Alphabetical;},
      }
    },
    SelectionMode::ISP => {
      match stats.isps_sort {
        SortState::AlphabeticalRev => {
          stats.isps.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.isps.items.reverse();
          stats.isps_sort = SortState::Alphabetical;},
        SortState::Alphabetical => {
          stats.isps.items.sort_by(|a, b|
            a.0.name.partial_cmp(&b.0.name).unwrap());
          stats.isps_sort = SortState::AlphabeticalRev;
        },
        _ => {stats.isps_sort = SortState::Alphabetical;},
      }
    },
    SelectionMode::IP => {
      match stats.ips_sort {
        SortState::AlphabeticalRev => {
          stats.ips.items.sort_by(|a, b|
            a.ip.partial_cmp(&b.ip).unwrap());
          stats.ips.items.reverse();
          stats.ips_sort = SortState::Alphabetical;},
        SortState::Alphabetical => {
          stats.ips.items.sort_by(|a, b|
            a.ip.partial_cmp(&b.ip).unwrap());
          stats.ips_sort = SortState::AlphabeticalRev;
        },
        _ => {stats.ips_sort = SortState::Alphabetical;},
      }
    },
  };
  Ok(())
}

// SORT NUM WARN// --------------------------------------------------------------- //
pub fn sort_by_numwarn(stats: &mut Stats) -> Result<()> {
  let _ =  match stats.selection_mode {
    SelectionMode::Country => {
      match stats.countries_sort {
        SortState::NumWarnsRev => {
          stats.countries.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.countries.items.reverse();
          stats.countries_sort = SortState::NumWarns;},
        SortState::NumWarns => {
          stats.countries.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.countries_sort = SortState::NumWarnsRev;
        },
        _ => {stats.countries_sort = SortState::NumWarns;},
      }
    },
    SelectionMode::Region => {
      match stats.regions_sort {
        SortState::NumWarnsRev => {
          stats.regions.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.regions.items.reverse();
          stats.regions_sort = SortState::NumWarns;},
        SortState::NumWarns => {
          stats.regions.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.regions_sort = SortState::NumWarnsRev;
        },
        _ => {stats.regions_sort = SortState::NumWarns;},
      }
    },
    SelectionMode::City => {
      match stats.cities_sort {
        SortState::NumWarnsRev => {
          stats.cities.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.cities.items.reverse();
          stats.cities_sort = SortState::NumWarns;},
        SortState::NumWarns => {
          stats.cities.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.cities_sort = SortState::NumWarnsRev;
        },
        _ => {stats.cities_sort = SortState::NumWarns;},
      }
    },
    SelectionMode::ISP => {
      match stats.isps_sort {
        SortState::NumWarnsRev => {
          stats.isps.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.isps.items.reverse();
          stats.isps_sort = SortState::NumWarns;},
        SortState::NumWarns => {
          stats.isps.items.sort_by(|a, b|
            a.0.warnings.partial_cmp(&b.0.warnings).unwrap());
          stats.isps_sort = SortState::NumWarnsRev;
        },
        _ => {stats.isps_sort = SortState::NumWarns;},
      }
    },
    SelectionMode::IP => {
      match stats.ips_sort {
        SortState::NumWarnsRev => {
          stats.ips.items.sort_by(|a, b|
            a.warnings.partial_cmp(&b.warnings).unwrap());
          stats.ips.items.reverse();
          stats.ips_sort = SortState::NumWarns;},
        SortState::NumWarns => {
          stats.ips.items.sort_by(|a, b|
            a.warnings.partial_cmp(&b.warnings).unwrap());
          stats.ips_sort = SortState::NumWarnsRev;
        },
        _ => {stats.ips_sort = SortState::NumWarns;},
      }
    },
  };
  Ok(())
}

// SORT BLOCKED // --------------------------------------------------------------- //
pub fn sort_by_blocked(stats: &mut Stats) -> Result<()> {
  let _ =  match stats.selection_mode {
    SelectionMode::Country => {
      match stats.countries_sort {
        SortState::BlockedRev => {
          stats.countries.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.countries.items.reverse();
          stats.countries_sort = SortState::Blocked;},
        SortState::Blocked => {
          stats.countries.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.countries_sort = SortState::BlockedRev;
        },
        _ => {stats.countries_sort = SortState::Blocked;},
      }
    },
    SelectionMode::Region => {
      match stats.regions_sort {
        SortState::BlockedRev => {
          stats.regions.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.regions.items.reverse();
          stats.regions_sort = SortState::Blocked;},
        SortState::Blocked => {
          stats.regions.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.regions_sort = SortState::BlockedRev;
        },
        _ => {stats.regions_sort = SortState::Blocked;},
      }
    },
    SelectionMode::City => {
      match stats.cities_sort {
        SortState::BlockedRev => {
          stats.cities.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.cities.items.reverse();
          stats.cities_sort = SortState::Blocked;},
        SortState::Blocked => {
          stats.cities.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.cities_sort = SortState::BlockedRev;
        },
        _ => {stats.cities_sort = SortState::Blocked;},
      }
    },
    SelectionMode::ISP => {
      match stats.isps_sort {
        SortState::BlockedRev => {
          stats.isps.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.isps.items.reverse();
          stats.isps_sort = SortState::Blocked;},
        SortState::Blocked => {
          stats.isps.items.sort_by(|a, b|
            a.0.is_blocked.partial_cmp(&b.0.is_blocked).unwrap());
          stats.isps_sort = SortState::BlockedRev;
        },
        _ => {stats.isps_sort = SortState::Blocked;},
      }
    },
    SelectionMode::IP => {
      match stats.ips_sort {
        SortState::BlockedRev => {
          stats.ips.items.sort_by(|a, b|
            a.warnings.partial_cmp(&b.warnings).unwrap());
          stats.ips.items.reverse();
          stats.ips_sort = SortState::Blocked;},
        SortState::Blocked => {
          stats.ips.items.sort_by(|a, b|
            a.warnings.partial_cmp(&b.warnings).unwrap());
          stats.ips_sort = SortState::BlockedRev;
        },
        _ => {stats.ips_sort = SortState::Blocked;},
      }
    },
  };
  Ok(())
}
