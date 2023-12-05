use std::{collections::HashMap, time::Duration};

use color_eyre::eyre::Result;
use crossterm::event::{KeyCode, KeyEvent};
use log::error;
use ratatui::{prelude::*, widgets::*};
use tokio::sync::mpsc::UnboundedSender;
use tracing::trace;
use tui_input::{backend::crossterm::EventHandler, Input};

use chrono::{self, Datelike};

use super::{Component, Frame};
use crate::{action::Action, config::key_event_to_string};

use crate::{migrations::schema::{city::City, region::Region, isp::ISP, country::Country},
themes::Theme, gen_structs::StatefulList};





pub fn convert_strings_to_utc(strings:Vec<String>) -> Vec<chrono::DateTime<chrono::FixedOffset>> {
    let mut ts: Vec<chrono::DateTime<chrono::FixedOffset>> = vec![];
    for stampstr in strings {
        let cts: chrono::DateTime<chrono::FixedOffset> = chrono::DateTime::parse_from_rfc3339(&stampstr).unwrap();
        ts.push(cts);
    };
    ts.sort_by(|a, b| a.cmp(b));
    ts
  }


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SelectionMode {
    #[default]
    Country,
    Region,
    City,
    ISP,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DrawMode {
    #[default]
    Static,
    Switching,
}


#[derive(Default)]
pub struct Stats {
  pub show_help: bool,
  pub counter: usize,
  pub app_ticker: usize,
  pub render_ticker: usize,
  pub mode: Mode,
  pub selection_mode: SelectionMode,
  pub drawmode: DrawMode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub text: Vec<String>,
  pub last_events: Vec<KeyEvent>,

  
  pub countries: StatefulList<(Country, Vec<chrono::DateTime<chrono::FixedOffset>>)>,
  pub regions: StatefulList<(Region, Vec<chrono::DateTime<chrono::FixedOffset>>)>,
  pub cities: StatefulList<(City, Vec<chrono::DateTime<chrono::FixedOffset>>)>,
  pub isps: StatefulList<(ISP, Vec<chrono::DateTime<chrono::FixedOffset>>)>,

  pub showing_stats: bool,
  pub apptheme: Theme,
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

    this
  }

  pub fn keymap(mut self, keymap: HashMap<KeyEvent, Action>) -> Self {
    self.keymap = keymap;
    self
  }

  pub fn tick(&mut self) {
    log::info!("Tick");
    self.app_ticker = self.app_ticker.saturating_add(1);
    self.last_events.drain(..);
  }

  pub fn render_tick(&mut self) {
    log::debug!("Render Tick");
    self.render_ticker = self.render_ticker.saturating_add(1);
  }

  pub fn add(&mut self, s: String) {
    self.text.push(s)
  }


  pub fn make_country_list(&mut self) -> List<'a> {

    let av_countries: Vec<ListItem> = self
    .countries
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.0.name.clone());   
        ListItem::new(line).style(self.apptheme.default_text_style)
    })
    .collect();
    // Create a List from all list items and highlight the currently selected one
    let countrylist: List<'_> = List::new(av_countries)
        .bg(self.apptheme.colors.lblack)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.selection_mode {
            SelectionMode::Country => {self.apptheme.active_border_style},
            _ => {self.apptheme.border_style},
          })
        .title("Countries"))
        .highlight_style(self.apptheme.highlight_item_style)
        .highlight_symbol(">> ");

    countrylist
  }

  pub fn make_region_list(&mut self) -> List<'a> {

    let av_regions: Vec<ListItem> = self
    .regions
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.0.name.clone());   
        ListItem::new(line).style(self.apptheme.default_text_style)
    })
    .collect();
    // Create a List from all list items and highlight the currently selected one
    let regionlist: List<'_> = List::new(av_regions)
        .bg(self.apptheme.colors.lblack)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.selection_mode {
            SelectionMode::Region => {self.apptheme.active_border_style},
            _ => {self.apptheme.border_style},
          })
        .title("Regions"))
        .highlight_style(self.apptheme.highlight_item_style)
        .highlight_symbol(">> ");

    regionlist
  }

  pub fn make_city_list(&mut self) -> List<'a> {

    let av_cities: Vec<ListItem> = self
    .cities
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.0.name.clone());   
        ListItem::new(line).style(self.apptheme.default_text_style)
    })
    .collect();
    // Create a List from all list items and highlight the currently selected one
    let citylist: List<'_> = List::new(av_cities)
        .bg(self.apptheme.colors.lblack)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.selection_mode {
            SelectionMode::City => {self.apptheme.active_border_style},
            _ => {self.apptheme.border_style},
          })
        .title("Cities"))
        .highlight_style(self.apptheme.highlight_item_style)
        .highlight_symbol(">> ");

    citylist
  }

  pub fn make_isp_list(&mut self) -> List<'a> {

    let av_isps: Vec<ListItem> = self
    .isps
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.0.name.clone());   
        ListItem::new(line).style(self.apptheme.default_text_style)
    })
    .collect();
    // Create a List from all list items and highlight the currently selected one
    let isplist: List<'_> = List::new(av_isps)
        .bg(self.apptheme.colors.lblack)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.selection_mode {
            SelectionMode::ISP => {self.apptheme.active_border_style},
            _ => {self.apptheme.border_style},
          })
        .title("ISPs"))
        .highlight_style(self.apptheme.highlight_item_style)
        .highlight_symbol(">> ");

    isplist
  }

  pub fn make_bars_for_timestamps(&mut self, timestamps: Vec<chrono::DateTime<chrono::FixedOffset>>) -> Vec<Bar<'a>> {
    let mut babars: Vec<Bar> = vec![];
    let mut aday:chrono::DateTime<chrono::FixedOffset> = timestamps[0];
    let mut num_aday: usize = 0; 
    let mut color_switcher: bool = false;
    for stamp in timestamps {
        let day_diff = stamp.day() - aday.day();
        if day_diff > 0 {
            // new bar
            let abar = Bar::default()
                .label(aday.format("%d/%m").to_string().into())
                .value(num_aday as u64)
                .style(if color_switcher {color_switcher = false; self.apptheme.username_style} else {color_switcher = true; self.apptheme.fail2ban_bg} )
                .value_style(if color_switcher {color_switcher = false; self.apptheme.default_text_style} else {color_switcher = true; self.apptheme.highlight_item_style});
            babars.push(abar);
            num_aday = 0;
            aday = stamp;

        } else {
            // add to old bar
            num_aday = num_aday.saturating_add(1);
        }
    }

    if babars.is_empty() {
        let abar = Bar::default()
        .label(aday.to_string().into())
        .value(num_aday as u64)
        .style(if color_switcher {color_switcher = false; self.apptheme.username_style} else {color_switcher = true; self.apptheme.fail2ban_bg} )
        .value_style(if color_switcher {color_switcher = false; self.apptheme.default_text_style} else {color_switcher = true; self.apptheme.highlight_item_style});  
        babars.push(abar);     
    }

    babars
  }

  pub fn create_barchart(&mut self, bars: Vec<Bar<'a>>, titlestr: &'a str) -> BarChart<'a> {
    let barchart = BarChart::default()
    .block(Block::default().title(titlestr).borders(Borders::ALL))
    .bar_width(10)
    .bar_gap(1)
    .group_gap(3)
    .bar_style(Style::new().yellow().on_red())
    .value_style(Style::new().red().bold())
    .label_style(Style::new().white())
    .data(BarGroup::default().bars(&bars))
    .max(100);
    barchart
  }

  pub fn make_country_overview(&mut self) -> impl Widget + '_ {
    // get totals
    let mut total_warn: u32 = 0;
    let mut total_banned: u32 = 0;
    let mut selected_warn: u32 = 0;
    let mut selected_banned: u32 = 0;
    let mut selected_warn_percent_total: f64 = 0.;
    let mut selected_ban_percent_total: f64= 0.;

    let mut paragraph = Paragraph::new(vec![]);

    let sel_idx = self.countries.state.selected();
    if sel_idx.is_some() {
        let sel_idx = sel_idx.unwrap();
        for i in 0..self.countries.items.len() {
            let tuple = self.countries.items[i].clone();
            total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
            total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));

        }
        let tuple = self.countries.items[sel_idx].clone();
        selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
        selected_banned = tuple.0.banned.try_into().unwrap_or(0);

        if selected_warn > 0 {
            selected_warn_percent_total = 100. / (f64::from(total_warn) / f64::from(selected_warn));
        } else {
            selected_warn_percent_total = 0. ;
        }
        if selected_banned > 0 {
            selected_ban_percent_total = 100. / (f64::from(total_banned) / f64::from(selected_banned));
        } else {
            selected_ban_percent_total = 0. ;
        }                
        
        let lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.0.name), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_warn_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_ban_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("Country Stats").bg(self.apptheme.colors.lblack))
  }

  pub fn make_region_overview(&mut self) -> impl Widget + '_ {
    // get totals
    let mut total_warn: u32 = 0;
    let mut total_banned: u32 = 0;
    let mut selected_warn: u32 = 0;
    let mut selected_banned: u32 = 0;
    let mut selected_warn_percent_total: f64 = 0.;
    let mut selected_ban_percent_total: f64= 0.;

    let mut paragraph = Paragraph::new(vec![]);

    let sel_idx = self.regions.state.selected();
    if sel_idx.is_some() {
        let sel_idx = sel_idx.unwrap();
        for i in 0..self.regions.items.len() {
            let tuple = self.regions.items[i].clone();
            total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
            total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));

        }
        let tuple = self.regions.items[sel_idx].clone();
        selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
        selected_banned = tuple.0.banned.try_into().unwrap_or(0);

        if selected_warn > 0 {
            selected_warn_percent_total = 100. / (f64::from(total_warn) / f64::from(selected_warn));
        } else {
            selected_warn_percent_total = 0. ;
        }
        if selected_banned > 0 {
            selected_ban_percent_total = 100. / (f64::from(total_banned) / f64::from(selected_banned));
        } else {
            selected_ban_percent_total = 0. ;
        }                
        
        let lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.0.name), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_warn_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_ban_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("Region Stats").bg(self.apptheme.colors.lblack)).bg(self.apptheme.colors.lblack)
  }

  pub fn make_city_overview(&mut self) -> impl Widget + '_ {
    // get totals
    let mut total_warn: u32 = 0;
    let mut total_banned: u32 = 0;
    let mut selected_warn: u32 = 0;
    let mut selected_banned: u32 = 0;
    let mut selected_warn_percent_total: f64 = 0.;
    let mut selected_ban_percent_total: f64= 0.;

    let mut paragraph = Paragraph::new(vec![]);

    let sel_idx = self.cities.state.selected();
    if sel_idx.is_some() {
        let sel_idx = sel_idx.unwrap();
        for i in 0..self.cities.items.len() {
            let tuple = self.cities.items[i].clone();
            total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
            total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));

        }
        let tuple = self.cities.items[sel_idx].clone();
        selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
        selected_banned = tuple.0.banned.try_into().unwrap_or(0);

        if selected_warn > 0 {
            selected_warn_percent_total = 100. / (f64::from(total_warn) / f64::from(selected_warn));
        } else {
            selected_warn_percent_total = 0. ;
        }
        if selected_banned > 0 {
            selected_ban_percent_total = 100. / (f64::from(total_banned) / f64::from(selected_banned));
        } else {
            selected_ban_percent_total = 0. ;
        }                
        
        let lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.0.name), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_warn_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_ban_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("City Stats").bg(self.apptheme.colors.lblack))
  }

  pub fn make_isp_overview(&mut self) -> impl Widget + '_ {
    // get totals
    let mut total_warn: u32 = 0;
    let mut total_banned: u32 = 0;
    let mut selected_warn: u32 = 0;
    let mut selected_banned: u32 = 0;
    let mut selected_warn_percent_total: f64 = 0.;
    let mut selected_ban_percent_total: f64= 0.;

    let mut paragraph = Paragraph::new(vec![]);

    let sel_idx = self.isps.state.selected();
    if sel_idx.is_some() {
        let sel_idx = sel_idx.unwrap();
        for i in 0..self.isps.items.len() {
            let tuple = self.isps.items[i].clone();
            total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
            total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));

        }
        let tuple = self.isps.items[sel_idx].clone();
        selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
        selected_banned = tuple.0.banned.try_into().unwrap_or(0);

        if selected_warn > 0 {
            selected_warn_percent_total = 100. / (f64::from(total_warn) / f64::from(selected_warn));
        } else {
            selected_warn_percent_total = 0. ;
        }
        if selected_banned > 0 {
            selected_ban_percent_total = 100. / (f64::from(total_banned) / f64::from(selected_banned));
        } else {
            selected_ban_percent_total = 0. ;
        }                
        
        let lines: Vec<Line> = vec![
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.0.name), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_warn_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {}", selected_ban_percent_total), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("ISP Stats").bg(self.apptheme.colors.lblack))
  }


}

impl Component for Stats {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    
    self.last_events.push(key.clone());
    let mut action = Action::Blank;
    if self.showing_stats {
        match self.mode {
            Mode::Insert | Mode::Processing => return Ok(None),
            Mode::Normal => {
              match key.code {
                KeyCode::Esc => {return Ok(Some(Action::StatsHide));},
                KeyCode::Char(keychar) => {
                    match keychar {
                        'E'|'e' => {return Ok(Some(Action::StatsHide));},
                        _ => {return Ok(Option::None);},
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
                    KeyCode::Up => {self.countries.previous();},
                    KeyCode::Down => {self.countries.next();},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::Region;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {return Ok(Some(Action::StatsGetCountries))},
                            _ => {},
                        }
                    },
                    _ => {},
                    }
            },
            SelectionMode::Region => {
                match key.code {
                    KeyCode::Up => {self.regions.previous();},
                    KeyCode::Down => {self.regions.next();},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::City;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {return Ok(Some(Action::StatsGetRegions))},
                            _ => {},
                        }
                    },
                    _ => {},
                    }
            },
            SelectionMode::City => {
                match key.code {
                KeyCode::Up => {self.cities.previous();},
                KeyCode::Down => {self.cities.next();},
                KeyCode::Tab => {self.selection_mode = SelectionMode::ISP;},
                KeyCode::Char(keychar) => {
                    match keychar {
                        'R'|'r' => {return Ok(Some(Action::StatsGetCities))},
                        _ => {},
                    }
                },
                _ => {},
                }
            },
            SelectionMode::ISP => {
                match key.code {
                    KeyCode::Up => {self.isps.previous();},
                    KeyCode::Down => {self.isps.next();},
                    KeyCode::Tab => {self.selection_mode = SelectionMode::Country;},
                    KeyCode::Char(keychar) => {
                        match keychar {
                            'R'|'r' => {return Ok(Some(Action::StatsGetISPs))},
                            _ => {},
                        }
                    },
                    _ => {},
                    }
            },
        }
    }

    Ok(Some(action))
  }

  fn update(&mut self, action: Action) -> Result<Option<Action>> {
        match action {
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
            Action::StartupDone => {self.countries.next(); self.regions.next(); self.cities.next(); self.isps.next();},

            Action::StatsGetCountries => {self.countries = StatefulList::with_items(vec![]);},
            Action::StatsGetRegions => {self.regions = StatefulList::with_items(vec![]);},
            Action::StatsGetCities => {self.cities = StatefulList::with_items(vec![]);},
            Action::StatsGetISPs => {self.isps = StatefulList::with_items(vec![]);},

            Action::StatsGotCountry(x, y) => {self.countries.items.push((x, convert_strings_to_utc(y)));},
            Action::StatsGotRegion(x, y) => {self.regions.items.push((x, convert_strings_to_utc(y)));},
            Action::StatsGotCity(x, y) => {self.cities.items.push((x, convert_strings_to_utc(y)));},
            Action::StatsGotISP(x, y) => {self.isps.items.push((x, convert_strings_to_utc(y)));},
                
            _ => (),
        }

    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {

    if self.showing_stats {
        let layout_a = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(rect);
        let layout_left = Layout::default().constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)].as_ref()).direction(Direction::Vertical).split(layout_a[0]);
        let layout_right = Layout::default().constraints([Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25), Constraint::Percentage(25)].as_ref()).direction(Direction::Vertical).split(layout_a[1]);

        let layout_country = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[0]);
        let layout_region = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[1]);
        let layout_city = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[2]);
        let layout_isp = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[3]);

        let countrylist = self.make_country_list();
        let regionlist = self.make_region_list();
        let citylist = self.make_city_list();
        let isplist = self.make_isp_list();

        // timestamp chart == barchart -> bar for every day with number of messages 
        let sel_country = self.countries.state.selected();
        if sel_country.is_some() {
            let bars = self.make_bars_for_timestamps(self.countries.items[sel_country.unwrap()].clone().1);
            let dtbars_country = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_country, layout_country[1]);

            let overview = self.make_country_overview();
            f.render_widget(overview, layout_country[0]);
        }

        let sel_region = self.regions.state.selected();
        if sel_region.is_some() {
            let bars = self.make_bars_for_timestamps(self.regions.items[sel_region.unwrap()].clone().1);
            let dtbars_region = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_region, layout_region[1]);

            let overview = self.make_region_overview();
            f.render_widget(overview, layout_region[0]);
        }    

        let sel_city = self.cities.state.selected();
        if sel_city.is_some() {
            let bars = self.make_bars_for_timestamps(self.cities.items[sel_city.unwrap()].clone().1);
            let dtbars_city = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_city, layout_city[1]);

            let overview = self.make_city_overview();
            f.render_widget(overview, layout_city[0]);
        }   

        let sel_isp = self.isps.state.selected();
        if sel_isp.is_some() {
            let bars = self.make_bars_for_timestamps(self.isps.items[sel_isp.unwrap()].clone().1);
            let dtbars_isp = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_isp, layout_isp[1]);

            let overview = self.make_isp_overview();
            f.render_widget(overview, layout_isp[0]);
        }

        f.render_stateful_widget(countrylist, layout_left[0], &mut self.countries.state);
        f.render_stateful_widget(regionlist, layout_left[1], &mut self.regions.state);
        f.render_stateful_widget(citylist, layout_left[2], &mut self.cities.state);
        f.render_stateful_widget(isplist, layout_left[3], &mut self.isps.state);

    }



    Ok(())
  }
}


