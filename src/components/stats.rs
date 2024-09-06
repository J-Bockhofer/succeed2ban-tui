pub mod ui;
use ui::Timeframe;

pub mod enums;
use enums::{Mode, DisplayMode, DrawMode, SelectionMode, SortMode, SortState, BlockMode};

pub mod utils;
use utils::{convert_strings_to_utc, get_msgs_per_ip};

pub mod actions;
use actions::refresh_countries;

use std::fmt::UpperExp;
use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use chrono::{self, Datelike};

use super::{Component, Frame};
use crate::{action::{Action, StatAction}, config::{Config, KeyBindings, get_first_key_simple, get_first_key_by_action}, components::home::utils::centered_rect};

use crate::{database::schema::{city::City, region::Region, isp::ISP, country::Country, message::MiniMessage, ip::IP},
themes::Theme, gen_structs::StatefulList, themes::Themes};


#[derive(Default, Clone, PartialEq, Eq)]
pub struct StatIP {
  ip: String,
  timestamps: Vec<chrono::DateTime<chrono::FixedOffset>>,
  warnings: usize,
}


#[derive(Default)]
pub struct Stats {
  /// Controls if Stats are shown 
  pub showing_stats: bool,
  //
  pub counter: usize,
  pub app_ticker: usize,
  pub render_ticker: usize,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub last_events: Vec<KeyEvent>,
  config: Config,
  // 
  pub mode: Mode,
  pub selection_mode: SelectionMode,
  pub drawmode: DrawMode,
  pub display_mode: DisplayMode,
  pub block_mode: BlockMode,
  pub sort_mode: SortMode,
  pub selected_timeframe: Timeframe,
  //
  pub countries: StatefulList<(Country, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_regions: Vec<(Region, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_cities: Vec<(City, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_isps: Vec<(ISP, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  //
  pub regions: StatefulList<(Region, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub cities: StatefulList<(City, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub isps: StatefulList<(ISP, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub ips: StatefulList<StatIP>,
  pub selected_ip: IP,
  //
  pub countries_sort: SortState,
  pub regions_sort: SortState,
  pub cities_sort: SortState,
  pub isps_sort: SortState,
  pub ips_sort: SortState,
  //
  pub apptheme: Theme,
  pub available_themes: Themes,

  
}

impl <'a> Stats  {
  pub fn new() -> Self {
    Self::default()
  }

  fn set_default_items() -> Self {
    let mut this = Self::default();
    this.showing_stats = false;
    this.countries = StatefulList::with_items(vec![]);
    this.regions = StatefulList::with_items(vec![]);
    this.cities = StatefulList::with_items(vec![]);
    this.isps = StatefulList::with_items(vec![]);
    this.ips = StatefulList::with_items(vec![]);

    this.full_regions = vec![];
    this.full_cities = vec![];
    this.full_isps = vec![];
    this.available_themes = Themes::default();

    this
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  pub fn tick(&mut self) {
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn make_ip_overview_old(&mut self) -> impl Widget + '_ {
    // get totals
    let selected_warn: u32;

    let mut paragraph = Paragraph::new(vec![]);

    let sel_idx = self.ips.state.selected();
    if sel_idx.is_some() {
        let sel_idx = sel_idx.unwrap();

        let tuple = self.ips.items[sel_idx].clone();
        selected_warn = tuple.warnings.try_into().unwrap_or(0);
                     
        let lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors_app.background_darkest.color)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.ip), Style::default().fg(self.apptheme.colors_app.accent_color_a_var.color))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), Style::default().fg(self.apptheme.colors_app.text_color.color))]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("IP Stats").bg(self.apptheme.colors_app.background_darkest.color))
  }

  pub fn selected_country(&mut self) {
    // find selected country
    let sel_idx = self.countries.state.selected();
    if sel_idx.is_none() || self.countries.items.len() < 1 {return;}
    let sel_idx = sel_idx.unwrap();
    let sel_country = self.countries.items[sel_idx].0.clone();
    let sel_country_ips = self.countries.items[sel_idx].2.clone();
    self.regions.unselect();
    self.regions = StatefulList::with_items(vec![]);
    // find all regions that have selected country and add them to the StatefulList
    for tuple in self.full_regions.clone().into_iter() {
      if tuple.0.country == sel_country.name {self.regions.items.push(tuple);}
    }
    // do same for isps
    self.isps.unselect();
    self.isps = StatefulList::with_items(vec![]);
    // find all regions that have selected country and add them to the StatefulList
    for tuple in self.full_isps.clone().into_iter() {
      if tuple.0.country == sel_country.name {self.isps.items.push(tuple);}
    }    
    self.isps.next();

    // do same for ips
    self.ips.unselect();
    self.ips = StatefulList::with_items(sel_country_ips);   
    self.ips.next();
    self.selected_ip();

    self.regions.next();
    self.selected_region();
  }

  pub fn selected_region(&mut self) {
    // find selected region
    let sel_idx = self.regions.state.selected();
    if sel_idx.is_none() || self.regions.items.len() < 1{return;}
    let sel_idx = sel_idx.unwrap();
    let sel_region = self.regions.items[sel_idx].0.clone();
    let sel_region_ips = self.regions.items[sel_idx].2.clone();
    self.cities.unselect();
    self.cities = StatefulList::with_items(vec![]);
    // find all cities that have selected region and add them to the StatefulList
    for tuple in self.full_cities.clone().into_iter() {
      if tuple.0.region == sel_region.name {self.cities.items.push(tuple);}
    }
    self.cities.next();

    // do same for ips
    self.ips.unselect();
    self.ips = StatefulList::with_items(sel_region_ips);   
    self.ips.next();
    self.selected_ip();
  }

  pub fn selected_city(&mut self) {
    // find selected region
    let sel_idx = self.cities.state.selected();
    if sel_idx.is_none() || self.cities.items.len() < 1{return;}
    let sel_idx = sel_idx.unwrap();
    let sel_city_ips = self.cities.items[sel_idx].2.clone();

    self.ips.unselect();
    self.ips = StatefulList::with_items(sel_city_ips);   
    self.ips.next();
    self.selected_ip();
  }

  pub fn selected_isp(&mut self) {
    // find selected region
    let sel_idx = self.isps.state.selected();
    if sel_idx.is_none() || self.isps.items.len() < 1{return;}
    let sel_idx = sel_idx.unwrap();
    let sel_isp_ips = self.isps.items[sel_idx].2.clone();

    self.ips.unselect();
    self.ips = StatefulList::with_items(sel_isp_ips);   
    self.ips.next();
    self.selected_ip();
  }

  pub fn selected_ip(&mut self) {
    // find selected ip
    let sel_idx = self.ips.state.selected();
    if sel_idx.is_none() || self.ips.items.len() < 1{return;}
    let sel_idx = sel_idx.unwrap();
    let sel_ip = self.ips.items[sel_idx].clone().ip;
    let tx = self.action_tx.clone().unwrap();
    tx.send(Action::StatsGetIP(sel_ip)).expect("Failed to reuest IP from Stats");
  }

  pub fn get_timestamps_from_msgs(&self, msgs: Vec<MiniMessage>) -> Vec<String> {
    let mut timestamps: Vec<String> = vec![];
    if msgs.is_empty() {return timestamps;}
    for msg in msgs {
      timestamps.push(msg.created_at);
    }
    timestamps
  }

  pub fn block_by_selected_mode(&mut self) -> Result<()> {

    match self.block_mode {
      BlockMode::Block => {
        match self.selection_mode {
          SelectionMode::Country => {actions::block_selected_country(self)?;},
          SelectionMode::Region => {actions::block_selected_region(self)?;},
          SelectionMode::City => {actions::block_selected_city(self)?;},
          SelectionMode::ISP => {actions::block_selected_isp(self)?;},
          SelectionMode::IP => {actions::block_selected_ip(self)?;},
        }
      },
      BlockMode::Unblock => {
        match self.selection_mode {
          SelectionMode::Country => {actions::unblock_selected_country(self)?;},
          SelectionMode::Region => {actions::unblock_selected_region(self)?;},
          SelectionMode::City => {actions::unblock_selected_city(self)?;},
          SelectionMode::ISP => {actions::unblock_selected_isp(self)?;},
          SelectionMode::IP => {actions::unblock_selected_ip(self)?;},
        }
      },
    }

    Ok(())
  }

  pub fn sort_by_selected_mode(&mut self) -> Result<()> {

    let _ = match self.sort_mode {
      SortMode::Alphabetical => {actions::sort_by_alphabetical(self)?;},
      SortMode::NumWarns => {actions::sort_by_numwarn(self)?;},
      SortMode::Blocked => {actions::sort_by_blocked(self)?;},
    };
    Ok(())
  }

  pub fn select_new_theme(&mut self, theme_name: String) {
    for themecontainer in self.available_themes.theme_collection.clone() {
      if themecontainer.name == theme_name {
        self.apptheme = themecontainer.theme;
        break;
      }
    }
  }

  pub fn previous_timeframe(&mut self) {
    match self.selected_timeframe {
      Timeframe::Day => {self.selected_timeframe = Timeframe::Year;},
      Timeframe::Week => {self.selected_timeframe = Timeframe::Day;},
      Timeframe::Month => {self.selected_timeframe = Timeframe::Week;},
      Timeframe::Year => {self.selected_timeframe = Timeframe::Month;},
    }
  }

  pub fn next_timeframe(&mut self) {
    match self.selected_timeframe {
      Timeframe::Day => {self.selected_timeframe = Timeframe::Week;},
      Timeframe::Week => {self.selected_timeframe = Timeframe::Month;},
      Timeframe::Month => {self.selected_timeframe = Timeframe::Year;},
      Timeframe::Year => {self.selected_timeframe = Timeframe::Day;},
    }
  }


}

impl Component for Stats {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn register_config_handler(&mut self, config: Config) -> Result<()> {
    self.config = config;
    Ok(())
  }


  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    
    self.last_events.push(key.clone());
    let action = Action::Blank;
    if self.showing_stats {
        match self.mode {
            Mode::Processing => return Ok(None),
            Mode::Normal => {
              match key.code {
                KeyCode::Esc => {return Ok(Some(Action::StatsHide));},
                _ => {
                  self.input.handle_event(&crossterm::event::Event::Key(key));
                },
              }
            },
            Mode::Block => {
              match key.code {
                KeyCode::Esc => {return Ok(Some(Action::StatsHide));},
                KeyCode::Char(keychar) => {
                    match keychar {
                        'Y'|'y' => {self.block_by_selected_mode()?; self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal;},
                        'N'|'n' => {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal;},
                        _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                    }
                }
                _ => {
                  self.input.handle_event(&crossterm::event::Event::Key(key));
                },
              }
            },            
        };

        match self.selection_mode {
            SelectionMode::Country => {
                match key.code {
                    KeyCode::Up => {self.countries.previous(); self.selected_country();},
                    KeyCode::Down => {self.countries.next(); self.selected_country();},
                    KeyCode::BackTab => {self.selection_mode = SelectionMode::IP;},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::Region;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {refresh_countries(self.action_tx.clone().unwrap())?},
                            _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                        }
                    },
                    _ => {},
                    }
            },
            SelectionMode::Region => {
                match key.code {
                    KeyCode::Up => {self.regions.previous(); self.selected_region();},
                    KeyCode::Down => {self.regions.next(); self.selected_region();},
                    KeyCode::BackTab => {self.selection_mode = SelectionMode::Country;},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::City;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {return Ok(Some(Action::StatsGetRegions))},
                            _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                        }
                    },
                    _ => {},
                    }
            },
            SelectionMode::City => {
                match key.code {
                KeyCode::Up => {self.cities.previous(); self.selected_city();},
                KeyCode::Down => {self.cities.next(); self.selected_city();},
                KeyCode::BackTab => {self.selection_mode = SelectionMode::Region;},
                KeyCode::Tab => {self.selection_mode = SelectionMode::ISP;},
                KeyCode::Char(keychar) => {
                    match keychar {
                        'R'|'r' => {return Ok(Some(Action::StatsGetCities))},
                        _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                    }
                },
                _ => {},
                }
            },
            SelectionMode::ISP => {
                match key.code {
                    KeyCode::Up => {self.isps.previous(); self.selected_isp();},
                    KeyCode::Down => {self.isps.next(); self.selected_isp();},
                    KeyCode::BackTab => {self.selection_mode = SelectionMode::City;},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::IP;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {return Ok(Some(Action::StatsGetISPs))},
                            _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                        }
                    },
                    _ => {},
                    }
            },
            SelectionMode::IP => {
              match key.code {
                  KeyCode::Up => {self.ips.previous(); self.selected_ip();},
                  KeyCode::Down => {self.ips.next(); self.selected_ip();},
                  KeyCode::BackTab => {self.selection_mode = SelectionMode::ISP;},
                  KeyCode::Tab => {self.selection_mode = SelectionMode::Country;},
                  _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                  }
          },
        }
    }

    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
            Action::Help => {if self.display_mode == DisplayMode::Help {self.display_mode = DisplayMode::Normal;} else {self.display_mode = DisplayMode::Help;} return Ok(Some(Action::Render))},
            Action::StatsShow => {self.showing_stats = true;},
            Action::StatsHide => {self.showing_stats = false;},
            Action::Tick => self.tick(),
            Action::Render => self.render_tick(),
            Action::EnterNormal => {
              self.mode = Mode::Normal;
            },
            Action::EnterProcessing => {
              self.mode = Mode::Processing;
            },
            Action::ExitProcessing => {
              // TODO: Make this go to previous mode instead
              self.mode = Mode::Normal;
            },
            Action::StartupDone => {self.countries.next(); self.selected_country();},

            Action::Stats(x) => {
              match x {
                StatAction::SortAlphabetical => {self.sort_mode = SortMode::Alphabetical; self.sort_by_selected_mode()?;},
                StatAction::SortWarnings => {self.sort_mode = SortMode::NumWarns; self.sort_by_selected_mode()?;},
                StatAction::SortBlocked => {self.sort_mode = SortMode::Blocked; self.sort_by_selected_mode()?;},

                StatAction::ExitStats => {return Ok(Some(Action::StatsHide));}
                StatAction::Block => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Block;}},
                StatAction::Unblock => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Unblock;}},

                StatAction::PreviousTimeframe => {self.previous_timeframe();},
                StatAction::NextTimeframe => {self.next_timeframe();},
                
                _ => {}
              }
            }

            Action::StatsGetCountries => {self.countries.unselect(); self.countries = StatefulList::with_items(vec![]);},
            Action::StatsGetRegions => {self.regions.unselect(); self.regions = StatefulList::with_items(vec![]); self.full_regions = vec![];},
            Action::StatsGetCities => {self.cities.unselect(); self.cities = StatefulList::with_items(vec![]); self.full_cities = vec![];},
            Action::StatsGetISPs => {self.isps.unselect(); self.isps = StatefulList::with_items(vec![]); self.full_isps = vec![];},

            Action::StatsGotCountry(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = get_msgs_per_ip(y);
              self.countries.items.push((x, timestamps, statips));}, //self.countries.items.push((x, convert_strings_to_utc(y)));
            Action::StatsGotRegion(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = get_msgs_per_ip(y);
              self.full_regions.push((x, timestamps, statips));}, // self.regions.items.push((x, convert_strings_to_utc(y)));
            Action::StatsGotCity(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = get_msgs_per_ip(y);              
              self.full_cities.push((x, timestamps, statips));},
            Action::StatsGotISP(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = get_msgs_per_ip(y);              
              self.full_isps.push((x, timestamps, statips));},
            Action::StatsGotIP(x) => {self.selected_ip = x;},
            Action::SelectTheme(x) => {self.select_new_theme(x)},   
            _ => (),
        }

    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {

    if self.showing_stats {

        // paint the background
        let bg = Paragraph::default().style(Style::default().bg(self.apptheme.colors_app.background_mid.color));
        f.render_widget(bg, rect);

        let layout_a = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(rect);
        let layout_left = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref()).direction(Direction::Vertical).split(layout_a[0]);
        let layout_right = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref()).direction(Direction::Vertical).split(layout_a[1]);

        let layout_country = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[0]);
        let layout_region = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[1]);
        let layout_city = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[2]);
        let layout_isp = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[3]);
        let layout_ip = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[4]);

        let countrylist = ui::make_country_list(self);
        let regionlist = ui::make_region_list(self);
        let citylist = ui::make_city_list(self);
        let isplist = ui::make_isp_list(self);
        let iplist = ui::make_ip_list(self);

        // timestamp chart == barchart -> bar for every day with number of messages 
        let sel_country = self.countries.state.selected();
        if sel_country.is_some() && self.countries.items.len() > 0 {
            let bars = ui::make_bars_for_timestamps(&self.apptheme, self.countries.items[sel_country.unwrap()].clone().1, self.selected_timeframe);
            let titlestr = format!("Log entries per {}", match self.selected_timeframe {
              Timeframe::Day => {"Day"},
              Timeframe::Week => {"Week"},
              Timeframe::Month => {"Month"},
              Timeframe::Year => {"Year"},
            });
            let dtbars_country = ui::create_barchart(&self.apptheme, bars, &titlestr);
            f.render_widget(dtbars_country, layout_country[1]);

            let overview = ui::make_country_overview(self);
            f.render_widget(overview, layout_country[0]);
        }

        let sel_region = self.regions.state.selected();
        if sel_region.is_some() && self.regions.items.len() > 0{
            let bars = ui::make_bars_for_timestamps(&self.apptheme, self.regions.items[sel_region.unwrap()].clone().1, self.selected_timeframe);
            let dtbars_region = ui::create_barchart(&self.apptheme, bars, "Log entries per Day");
            f.render_widget(dtbars_region, layout_region[1]);

            let overview = ui::make_region_overview(self);
            f.render_widget(overview, layout_region[0]);
        }    

        let sel_city = self.cities.state.selected();
        if sel_city.is_some() && self.cities.items.len() > 0{
            let bars = ui::make_bars_for_timestamps(&self.apptheme, self.cities.items[sel_city.unwrap()].clone().1, self.selected_timeframe);
            let dtbars_city = ui::create_barchart(&self.apptheme, bars, "Log entries per Day");
            f.render_widget(dtbars_city, layout_city[1]);

            let overview = ui::make_city_overview(self);
            f.render_widget(overview, layout_city[0]);
        }   

        let sel_isp = self.isps.state.selected();
        if sel_isp.is_some() && self.isps.items.len() > 0{
            let bars = ui::make_bars_for_timestamps(&self.apptheme, self.isps.items[sel_isp.unwrap()].clone().1, self.selected_timeframe);
            let dtbars_isp = ui::create_barchart(&self.apptheme, bars, "Log entries per Day");
            f.render_widget(dtbars_isp, layout_isp[1]);

            let overview = ui::make_isp_overview(self);
            f.render_widget(overview, layout_isp[0]);
        }

        let sel_ip = self.ips.state.selected();
        if sel_ip.is_some() && self.ips.items.len() > 0 {
            let bars = ui::make_bars_for_timestamps(&self.apptheme, self.ips.items[sel_ip.unwrap()].clone().timestamps, self.selected_timeframe);
            let dtbars_ip = ui::create_barchart(&self.apptheme, bars, "Log entries per Day");
            f.render_widget(dtbars_ip, layout_ip[1]);

            let overview = ui::make_ip_overview(&self.apptheme, self.selected_ip.clone());
            f.render_widget(overview, layout_ip[0]);
        }        

        f.render_stateful_widget(countrylist, layout_left[0], &mut self.countries.state);
        f.render_stateful_widget(regionlist, layout_left[1], &mut self.regions.state);
        f.render_stateful_widget(citylist, layout_left[2], &mut self.cities.state);
        f.render_stateful_widget(isplist, layout_left[3], &mut self.isps.state);
        f.render_stateful_widget(iplist, layout_left[4], &mut self.ips.state);

        match self.display_mode {
          DisplayMode::Confirm => {
            let block_mode = if self.block_mode == BlockMode::Block {true} else {false};
            let p_area = centered_rect(f.size(), 40, 7);
            f.render_widget(Clear, p_area);
            f.render_widget(ui::popup_un_block_selected(self, block_mode),p_area);
          },
          DisplayMode::Help => {
            let p_area = centered_rect(f.size(), 35, 30);
            f.render_widget(Clear, p_area);
            f.render_widget(ui::popup_help(&self.apptheme, self.config.clone()),p_area);
            },
          _ => {}
        }
    }

    Ok(())
  }
}