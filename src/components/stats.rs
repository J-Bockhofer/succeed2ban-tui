use std::fmt::UpperExp;
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

use crate::{migrations::schema::{city::City, region::Region, isp::ISP, country::Country, message::MiniMessage, ip::IP},
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

#[derive(Default, Clone, PartialEq, Eq)]
pub struct StatIP {
  ip: String,
  timestamps: Vec<chrono::DateTime<chrono::FixedOffset>>,
  warnings: usize,
}


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  Insert,
  Processing,
  Block,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum BlockMode {
  #[default]
  Block,
  Unblock,
}


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SelectionMode {
    #[default]
    Country,
    Region,
    City,
    ISP,
    IP,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DrawMode {
    #[default]
    Static,
    Switching,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
    #[default]
    Normal,
    Help,
    Confirm,
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
  pub display_mode: DisplayMode,
  pub block_mode: BlockMode,
  pub input: Input,
  pub action_tx: Option<UnboundedSender<Action>>,
  pub keymap: HashMap<KeyEvent, Action>,
  pub text: Vec<String>,
  pub last_events: Vec<KeyEvent>,

  
  pub countries: StatefulList<(Country, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_regions: Vec<(Region, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_cities: Vec<(City, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub full_isps: Vec<(ISP, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,


  pub regions: StatefulList<(Region, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub cities: StatefulList<(City, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub isps: StatefulList<(ISP, Vec<chrono::DateTime<chrono::FixedOffset>>, Vec<StatIP>)>,
  pub ips: StatefulList<StatIP>,

  pub selected_ip: IP,

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
    this.ips = StatefulList::with_items(vec![]);

    this.full_regions = vec![];
    this.full_cities = vec![];
    this.full_isps = vec![];

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

  pub fn popup_un_block_selected(&mut self, is_block:bool)  -> impl Widget + '_  {
    let smode = self.selection_mode;

    let modestr = match smode {
      SelectionMode::Country =>{"Country"},
      SelectionMode::Region =>{"Region"},
      SelectionMode::City =>{"City"},
      SelectionMode::ISP =>{"ISP"},
      SelectionMode::IP =>{"IP"},
    };
    let sel_str = match smode {
      SelectionMode::Country =>{
        if self.countries.items.is_empty() {format!("")} else {
          let sel_idx = self.countries.state.selected().unwrap();
          let sel_item = self.countries.items[sel_idx].0.clone().name;
          sel_item
        }
      },
      SelectionMode::Region =>{
        if self.countries.items.is_empty() {format!("")} else {
          let sel_idx = self.regions.state.selected().unwrap();
          let sel_item = self.regions.items[sel_idx].0.clone().name;
          sel_item
        }
      },
      SelectionMode::City =>{
        if self.countries.items.is_empty() {format!("")} else {
          let sel_idx = self.cities.state.selected().unwrap();
          let sel_item = self.cities.items[sel_idx].0.clone().name;
          sel_item
        }
      },
      SelectionMode::ISP =>{
        if self.countries.items.is_empty() {format!("")} else {
          let sel_idx = self.isps.state.selected().unwrap();
          let sel_item = self.isps.items[sel_idx].0.clone().name;
          sel_item
        }
      },
      SelectionMode::IP =>{
        if self.countries.items.is_empty() {format!("")} else {
          let sel_idx = self.ips.state.selected().unwrap();
          let sel_item = self.ips.items[sel_idx].ip.clone();
          sel_item
        }
      },
    };

    let blockstr = if is_block {"BLOCK"} else {"UNBLOCK"};

    let mut clearlisttext: Vec<Line> = vec![];
    let clearlistline =   Line::from(vec![
      Span::styled(format!("Press "), self.apptheme.default_text_style), 
      Span::styled(format!("Y | y "), Style::default().fg(self.apptheme.colors.accent_lime)),
      Span::styled(format!("to confirm or "), self.apptheme.default_text_style),
      Span::styled(format!("N | n "), Style::default().fg(self.apptheme.colors.accent_orange)),
      Span::styled(format!("to cancel."), self.apptheme.default_text_style),
      ]);
    //clearlistline.patch_style(self.apptheme.selected_ip_bg);
    clearlisttext.push(Line::from(vec![Span::styled(format!(""), self.apptheme.default_text_style)]));
    clearlisttext.push(clearlistline);

    let clearlistbox = Paragraph::new(clearlisttext)
    .alignment(Alignment::Center)
    .set_style(Style::default())
    .block(Block::default()
    .bg(self.apptheme.colors.lblack)
    .borders(Borders::ALL)
    .title(format!("[ Confirm to {} all IPs by selected {} for {} ]", blockstr, modestr, sel_str)).style(self.apptheme.default_text_style).title_alignment(Alignment::Center));
    clearlistbox

  }

  pub fn centered_rect(&self, r: Rect, percent_x: u16, percent_y: u16) -> Rect {
    let popup_layout = Layout::default()
      .direction(Direction::Vertical)
      .constraints([
        Constraint::Percentage((100 - percent_y) / 2),
        Constraint::Percentage(percent_y),
        Constraint::Percentage((100 - percent_y) / 2),
      ])
      .split(r);
  
    Layout::default()
      .direction(Direction::Horizontal)
      .constraints([
        Constraint::Percentage((100 - percent_x) / 2),
        Constraint::Percentage(percent_x),
        Constraint::Percentage((100 - percent_x) / 2),
      ])
      .split(popup_layout[1])[1]
  }

  pub fn make_country_list(&mut self) -> List<'a> {

    let av_countries: Vec<ListItem> = self
    .countries
    .items
    .iter()
    .map(|i| {
        let is_blocked = i.0.is_blocked;
        let mut line = Line::from(format!("{}", i.0.name.clone()));
        line.patch_style(if is_blocked {self.apptheme.default_text_style.bg(self.apptheme.colors.accent_wred) } else { self.apptheme.default_text_style});
        ListItem::new(line)
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
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {self.apptheme.default_text_style.bg(self.apptheme.colors.accent_wred) } else { self.apptheme.default_text_style});
      ListItem::new(line)
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
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {self.apptheme.default_text_style.bg(self.apptheme.colors.accent_wred) } else { self.apptheme.default_text_style});
      ListItem::new(line)
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
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {self.apptheme.default_text_style.bg(self.apptheme.colors.accent_wred) } else { self.apptheme.default_text_style});
      ListItem::new(line)
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

  pub fn make_ip_list(&mut self) -> List<'a> {

    let av_ips: Vec<ListItem> = self
    .ips
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.ip.clone()); 
        ListItem::new(line).style(self.apptheme.default_text_style)
    })
    .collect();
    // Create a List from all list items and highlight the currently selected one
    let iplist: List<'_> = List::new(av_ips)
        .bg(self.apptheme.colors.lblack)
        .block(Block::default()
        .borders(Borders::ALL)
        .border_style( 
          match self.selection_mode {
            SelectionMode::IP => {self.apptheme.active_border_style},
            _ => {self.apptheme.border_style},
          })
        .title("IPs"))
        .highlight_style(self.apptheme.highlight_item_style)
        .highlight_symbol(">> ");

    iplist
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
                .style(if color_switcher {color_switcher = false; self.apptheme.username_style} else {color_switcher = true; self.apptheme.default_text_style.fg(self.apptheme.colors.accent_dblue)} )
                .value_style(self.apptheme.fail2ban_bg);
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
        .value_style(if color_switcher {self.apptheme.default_text_style} else {self.apptheme.highlight_item_style});  
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
    .bar_style(Style::new().yellow().bg(self.apptheme.colors.ddblue))
    .value_style(Style::new().white().bold())
    .label_style(Style::new().white())
    .data(BarGroup::default().bars(&bars))
    .max(100);
    barchart
  }

  pub fn make_country_overview(&mut self) -> impl Widget + '_ {
    // get totals
    let mut total_warn: u32 = 0;
    let mut total_banned: u32 = 0;
    let selected_warn: u32;
    let selected_banned: u32;
    let selected_warn_percent_total: f64;
    let selected_ban_percent_total: f64;
    let is_blocked: bool;

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
        is_blocked = tuple.0.is_blocked;

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
            Line::from(vec![ 
              if is_blocked {
                Span::styled(format!(" Blocked"), Style::default().fg(self.apptheme.colors.accent_orange))
              } else {
                Span::styled(format!(" Open   "), Style::default().fg(self.apptheme.colors.accent_lime))
              }
              ]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_warn_percent_total, prec=2), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_ban_percent_total, prec=2), self.apptheme.default_text_style)]),
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
    let selected_warn: u32;
    let selected_banned: u32;
    let selected_warn_percent_total: f64;
    let selected_ban_percent_total: f64;
    let is_blocked: bool;

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
        is_blocked = tuple.0.is_blocked;

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
            Line::from(vec![ 
              if is_blocked {
                Span::styled(format!(" Blocked"), Style::default().fg(self.apptheme.colors.accent_orange))
              } else {
                Span::styled(format!(" Open   "), Style::default().fg(self.apptheme.colors.accent_lime))
              }
              ]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_warn_percent_total, prec=2), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_ban_percent_total, prec=2), self.apptheme.default_text_style)]),
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
    let selected_warn: u32;
    let selected_banned: u32;
    let selected_warn_percent_total: f64;
    let selected_ban_percent_total: f64;
    let is_blocked: bool;

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
        is_blocked = tuple.0.is_blocked;

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
            Line::from(vec![ 
              if is_blocked {
                Span::styled(format!(" Blocked"), Style::default().fg(self.apptheme.colors.accent_orange))
              } else {
                Span::styled(format!(" Open   "), Style::default().fg(self.apptheme.colors.accent_lime))
              }
              ]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_warn_percent_total, prec=2), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_ban_percent_total, prec=2), self.apptheme.default_text_style)]),
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
    let selected_warn: u32;
    let selected_banned: u32;
    let selected_warn_percent_total: f64;
    let selected_ban_percent_total: f64;
    let is_blocked: bool;

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
        is_blocked = tuple.0.is_blocked;

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
            Line::from(vec![Span::styled(format!(" Selected     :"), self.apptheme.default_text_style.bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.0.name), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![ 
              if is_blocked {
                Span::styled(format!(" Blocked"), Style::default().fg(self.apptheme.colors.accent_orange))
              } else {
                Span::styled(format!(" Open   "), Style::default().fg(self.apptheme.colors.accent_lime))
              }
              ]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_warn_percent_total, prec=2), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), self.apptheme.default_text_style)]),

            Line::from(vec![Span::styled(format!(" Banned       : {}", selected_banned), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" % total      : {number:.prec$} %", number=selected_ban_percent_total, prec=2), self.apptheme.default_text_style)]),
            Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("ISP Stats").bg(self.apptheme.colors.lblack))
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
            Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
            Line::from(vec![Span::styled(format!(" {}", tuple.ip), Style::default().fg(self.apptheme.colors.accent_lorange))]),
            Line::from(vec![Span::styled(format!(" Warnings     : {}", selected_warn), self.apptheme.default_text_style)]),
        ];

        paragraph = Paragraph::new(lines);
    }
    paragraph.block(Block::default().borders(Borders::ALL).title("IP Stats").bg(self.apptheme.colors.lblack))
  }

  pub fn make_ip_overview(&mut self) -> impl Widget + '_ {
    // get totals
    if self.selected_ip == IP::default() {return Paragraph::new(vec![])}//self.make_ip_overview_old();}

    let selected_ip = self.selected_ip.clone();
    
    let ip = selected_ip.ip;
    let warn = selected_ip.warnings;
    let is_banned = selected_ip.is_banned;
    let banned_times = selected_ip.banned_times;
    let country = selected_ip.country;
    let city = selected_ip.city;
    let region = selected_ip.region;
    let isp = selected_ip.isp;

    //let mut paragraph = Paragraph::new(vec![]);

       
    let lines: Vec<Line> = vec![
        Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(self.apptheme.colors.lblack)), ]),
        Line::from(vec![Span::styled(format!(" {}", ip), Style::default().fg(self.apptheme.colors.accent_lorange))]),
        Line::from(vec![ 
          if is_banned {
            Span::styled(format!(" Banned  "), Style::default().fg(self.apptheme.colors.accent_orange))
          } else {
            Span::styled(format!(" Welcomed"), Style::default().fg(self.apptheme.colors.accent_lime))
          }
          ]),        
        Line::from(vec![Span::styled(format!(" Warnings     : {}", warn), self.apptheme.default_text_style)]),
        Line::from(vec![Span::styled(format!(" Banned times : {}", banned_times), self.apptheme.default_text_style)]),
        Line::from(vec![Span::styled(format!(" {city}, {region}, {country} ", ), self.apptheme.default_text_style)]),
        Line::from(vec![Span::styled(format!(" {isp} " ), self.apptheme.default_text_style)]),
    ];

    let paragraph = Paragraph::new(lines);

    paragraph.block(Block::default().borders(Borders::ALL).title("IP Stats").bg(self.apptheme.colors.lblack))
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


  pub fn refresh_countries(&mut self) -> Result<()> {
    let tx = self.action_tx.clone().unwrap();

    tokio::spawn(async move {
      tx.send(Action::StatsGetCountries).expect("Failed to refresh countries; E404");
      tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
      tx.send(Action::StatsGetRegions).expect("Failed to refresh regions; E404");
      tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
      tx.send(Action::StatsGetCities).expect("Failed to refresh cities; E404");
      tokio::time::sleep(tokio::time::Duration::from_millis(25)).await;
      tx.send(Action::StatsGetISPs).expect("Failed to refresh ISPs; E404");
    });
    
    Ok(())
  }

  pub fn get_msgs_per_ip(&self, msgs: Vec<MiniMessage>) -> Vec<StatIP> {
    let mut ipvec: Vec<StatIP> = vec![];
    if msgs.is_empty() {return ipvec;}
    let mut last_ip = msgs[0].ip.clone();
    let mut last_timestamps: Vec<String> = vec![];
    let mut num_this: usize = 0;
    for msg in msgs {
      let cip = msg.ip;
      if cip == last_ip {
        last_timestamps.push(msg.created_at);
        num_this = num_this.saturating_add(1);
      } else {
        let rt: Vec<chrono::DateTime<chrono::FixedOffset>> = convert_strings_to_utc(last_timestamps);
        let statip = StatIP{ip: last_ip, timestamps: rt, warnings: num_this};
        ipvec.push(statip);
        last_ip = cip;
        num_this = 0;
        last_timestamps = vec![];
      }    
    }

    if ipvec.is_empty() {
      let rt: Vec<chrono::DateTime<chrono::FixedOffset>> = convert_strings_to_utc(last_timestamps);
      let statip = StatIP{ip: last_ip, timestamps: rt, warnings: num_this};
      ipvec.push(statip);      
    }

    ipvec
  }

  pub fn get_timestamps_from_msgs(&self, msgs: Vec<MiniMessage>) -> Vec<String> {
    let mut timestamps: Vec<String> = vec![];
    if msgs.is_empty() {return timestamps;}
    for msg in msgs {
      timestamps.push(msg.created_at);
    }
    timestamps
  }


  pub fn block_selected_country(&mut self) -> Result<()> {
    if self.countries.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.countries.state.selected().unwrap();
    let sel_country = self.countries.items[sel_idx].clone().0;
    tx.send(Action::StatsBlockCountry(sel_country)).expect("Failed to send request to block Country");
    self.countries.items[sel_idx].0.is_blocked = true;
    Ok(())
  }

  pub fn unblock_selected_country(&mut self) -> Result<()> {
    if self.countries.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.countries.state.selected().unwrap();
    let sel_country = self.countries.items[sel_idx].clone().0;
    tx.send(Action::StatsUnblockCountry(sel_country)).expect("Failed to send request to unblock Country");
    self.countries.items[sel_idx].0.is_blocked = false;
    Ok(())
  }



  pub fn block_selected_region(&mut self) -> Result<()> {
    if self.regions.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.regions.state.selected().unwrap();
    let sel_region = self.regions.items[sel_idx].clone().0;
    tx.send(Action::StatsBlockRegion(sel_region)).expect("Failed to send request to block Region");
    self.regions.items[sel_idx].0.is_blocked = true;
    Ok(())
  }

  pub fn unblock_selected_region(&mut self) -> Result<()> {
    if self.regions.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.regions.state.selected().unwrap();
    let sel_region = self.regions.items[sel_idx].clone().0;
    tx.send(Action::StatsUnblockRegion(sel_region)).expect("Failed to send request to unblock Region");
    self.regions.items[sel_idx].0.is_blocked = false;
    Ok(())
  }


  pub fn block_selected_city(&mut self) -> Result<()> {
    if self.cities.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.cities.state.selected().unwrap();
    let sel_city = self.cities.items[sel_idx].clone().0;
    tx.send(Action::StatsBlockCity(sel_city)).expect("Failed to send request to block City");
    self.cities.items[sel_idx].0.is_blocked = true;
    Ok(())
  }

  pub fn unblock_selected_city(&mut self) -> Result<()> {
    if self.cities.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.cities.state.selected().unwrap();
    let sel_city = self.cities.items[sel_idx].clone().0;
    tx.send(Action::StatsUnblockCity(sel_city)).expect("Failed to send request to unblock City");
    self.cities.items[sel_idx].0.is_blocked = false;
    Ok(())
  }


  pub fn block_selected_isp(&mut self) -> Result<()> {
    if self.isps.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.isps.state.selected().unwrap();
    let sel_isp = self.isps.items[sel_idx].clone().0;
    tx.send(Action::StatsBlockISP(sel_isp)).expect("Failed to send request to block ISP");
    self.isps.items[sel_idx].0.is_blocked = true;
    Ok(())
  }

  pub fn unblock_selected_isp(&mut self) -> Result<()> {
    if self.isps.items.is_empty() {return Ok(())}
    let tx = self.action_tx.clone().unwrap();
    let sel_idx = self.isps.state.selected().unwrap();
    let sel_isp = self.isps.items[sel_idx].clone().0;
    tx.send(Action::StatsUnblockISP(sel_isp)).expect("Failed to send request to unblock ISP");
    self.isps.items[sel_idx].0.is_blocked = false;
    Ok(())
  }


  pub fn block_by_selected_mode(&mut self) -> Result<()> {
    let tx = self.action_tx.clone().unwrap();

    match self.block_mode {
      BlockMode::Block => {
        match self.selection_mode {
          SelectionMode::Country => {self.block_selected_country()?;},
          SelectionMode::Region => {self.block_selected_region()?;},
          SelectionMode::City => {self.block_selected_city()?;},
          SelectionMode::ISP => {self.block_selected_isp()?;},
          SelectionMode::IP => {todo!("self.block_selected_ip()")},
        }
      },
      BlockMode::Unblock => {
        match self.selection_mode {
          SelectionMode::Country => {self.unblock_selected_country()?;},
          SelectionMode::Region => {self.unblock_selected_region()?;},
          SelectionMode::City => {self.unblock_selected_city()?;},
          SelectionMode::ISP => {self.unblock_selected_isp()?;},
          SelectionMode::IP => {todo!("self.unblock_selected_ip()")},
        }
      }
    }

    Ok(())
  }

}

impl Component for Stats {
  fn register_action_handler(&mut self, tx: UnboundedSender<Action>) -> Result<()> {
    self.action_tx = Some(tx);
    Ok(())
  }

  fn handle_key_events(&mut self, key: KeyEvent) -> Result<Option<Action>> {
    
    self.last_events.push(key.clone());
    let action = Action::Blank;
    if self.showing_stats {
        match self.mode {
            Mode::Insert | Mode::Processing => return Ok(None),
            Mode::Normal => {
              match key.code {
                KeyCode::Esc => {return Ok(Some(Action::StatsHide));},
                KeyCode::Char(keychar) => {
                    match keychar {
                        'E'|'e' => {return Ok(Some(Action::StatsHide));},
                        'W'|'w' => {todo!("Help Screen")},
                        'B'|'b' => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Block;}},
                        'U'|'u' => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Unblock;}},
                        _ => {self.input.handle_event(&crossterm::event::Event::Key(key));},
                    }
                }
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
                        'E'|'e' => {return Ok(Some(Action::StatsHide));},
                        'W'|'w' => {todo!("Help Screen")},
                        'B'|'b' => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Block;}},
                        'U'|'u' => {if self.mode == Mode::Block {self.mode = Mode::Normal; self.display_mode = DisplayMode::Normal; } else {self.mode = Mode::Block; self.display_mode = DisplayMode::Confirm; self.block_mode = BlockMode::Unblock;}},
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
                            'R'|'r' => {self.refresh_countries()?},
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

            Action::StatsGetCountries => {self.countries.unselect(); self.countries = StatefulList::with_items(vec![]);},
            Action::StatsGetRegions => {self.regions.unselect(); self.regions = StatefulList::with_items(vec![]); self.full_regions = vec![];},
            Action::StatsGetCities => {self.cities.unselect(); self.cities = StatefulList::with_items(vec![]); self.full_cities = vec![];},
            Action::StatsGetISPs => {self.isps.unselect(); self.isps = StatefulList::with_items(vec![]); self.full_isps = vec![];},

            Action::StatsGotCountry(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = self.get_msgs_per_ip(y);
              self.countries.items.push((x, timestamps, statips));}, //self.countries.items.push((x, convert_strings_to_utc(y)));
            Action::StatsGotRegion(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = self.get_msgs_per_ip(y);
              self.full_regions.push((x, timestamps, statips));}, // self.regions.items.push((x, convert_strings_to_utc(y)));
            Action::StatsGotCity(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = self.get_msgs_per_ip(y);              
              self.full_cities.push((x, timestamps, statips));},
            Action::StatsGotISP(x, y) => {
              let timestamps = convert_strings_to_utc(self.get_timestamps_from_msgs(y.clone()));
              let statips = self.get_msgs_per_ip(y);              
              self.full_isps.push((x, timestamps, statips));},

            Action::StatsGotIP(x) => {self.selected_ip = x;}
                
            _ => (),
        }

    Ok(None)
  }

  fn draw(&mut self, f: &mut Frame<'_>, rect: Rect) -> Result<()> {

    if self.showing_stats {

        // paint the background
        let bg = Paragraph::default().style(self.apptheme.default_background);
        f.render_widget(bg, rect);

        let layout_a = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(rect);
        let layout_left = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref()).direction(Direction::Vertical).split(layout_a[0]);
        let layout_right = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20), Constraint::Percentage(20)].as_ref()).direction(Direction::Vertical).split(layout_a[1]);

        let layout_country = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[0]);
        let layout_region = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[1]);
        let layout_city = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[2]);
        let layout_isp = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[3]);
        let layout_ip = Layout::default().constraints([Constraint::Percentage(20), Constraint::Percentage(80)].as_ref()).direction(Direction::Horizontal).split(layout_right[4]);

        let countrylist = self.make_country_list();
        let regionlist = self.make_region_list();
        let citylist = self.make_city_list();
        let isplist = self.make_isp_list();
        let iplist = self.make_ip_list();

        // timestamp chart == barchart -> bar for every day with number of messages 
        let sel_country = self.countries.state.selected();
        if sel_country.is_some() && self.countries.items.len() > 0 {
            let bars = self.make_bars_for_timestamps(self.countries.items[sel_country.unwrap()].clone().1);
            let dtbars_country = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_country, layout_country[1]);

            let overview = self.make_country_overview();
            f.render_widget(overview, layout_country[0]);
        }

        let sel_region = self.regions.state.selected();
        if sel_region.is_some() && self.regions.items.len() > 0{
            let bars = self.make_bars_for_timestamps(self.regions.items[sel_region.unwrap()].clone().1);
            let dtbars_region = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_region, layout_region[1]);

            let overview = self.make_region_overview();
            f.render_widget(overview, layout_region[0]);
        }    

        let sel_city = self.cities.state.selected();
        if sel_city.is_some() && self.cities.items.len() > 0{
            let bars = self.make_bars_for_timestamps(self.cities.items[sel_city.unwrap()].clone().1);
            let dtbars_city = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_city, layout_city[1]);

            let overview = self.make_city_overview();
            f.render_widget(overview, layout_city[0]);
        }   

        let sel_isp = self.isps.state.selected();
        if sel_isp.is_some() && self.isps.items.len() > 0{
            let bars = self.make_bars_for_timestamps(self.isps.items[sel_isp.unwrap()].clone().1);
            let dtbars_isp = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_isp, layout_isp[1]);

            let overview = self.make_isp_overview();
            f.render_widget(overview, layout_isp[0]);
        }

        let sel_ip = self.ips.state.selected();
        if sel_ip.is_some() && self.ips.items.len() > 0{
            let bars = self.make_bars_for_timestamps(self.ips.items[sel_ip.unwrap()].clone().timestamps);
            let dtbars_ip = self.create_barchart(bars, "Log entries per Day");
            f.render_widget(dtbars_ip, layout_ip[1]);

            let overview = self.make_ip_overview();
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

            let p_area = self.centered_rect(f.size(), 40, 7);
            f.render_widget(Clear, p_area);
            f.render_widget(self.popup_un_block_selected(block_mode),p_area);
          },
          _ => {}
        }
    }



    Ok(())
  }
}


