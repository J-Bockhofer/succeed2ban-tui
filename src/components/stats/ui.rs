use super::{SelectionMode, SortState, Stats};
use crate::migrations::schema::{city::City, country::Country, ip::IP, isp::ISP, message::MiniMessage, region::Region};
use crate::{gen_structs::StatefulList, themes::Theme};
use chrono::{DateTime, Datelike, FixedOffset};
use color_eyre::owo_colors::OwoColorize;
use ratatui::widgets::block::Title;
use ratatui::{prelude::*, widgets::*};

// Rendered Lists     // ---------------------------------------------------------------- //

pub fn make_country_list<'a>(stats: &Stats) -> List<'a> {
  let av_countries: Vec<ListItem> = stats
    .countries
    .items
    .iter()
    .map(|i| {
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {
        Style::default().fg(stats.apptheme.colors_app.text_color.color).bg(stats.apptheme.colors_app.warn_color.color)
      } else {
        Style::default().fg(stats.apptheme.colors_app.text_color.color)
      });
      ListItem::new(line)
    })
    .collect();
  let mut sel_item = Country::default();
  let sel_idx = stats.countries.state.selected();
  if sel_idx.is_some() && !stats.countries.items.is_empty() {
    sel_item = stats.countries.items[sel_idx.unwrap()].0.clone();
  }

  // create sort indicator
  let sort_indicator = make_sort_state_indicator(&stats.apptheme, stats.countries_sort);
  // Create a List from all list items and highlight the currently selected one
  let countrylist: List<'_> = List::new(av_countries)
    .bg(stats.apptheme.colors_app.background_darkest.color)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(match stats.selection_mode {
          SelectionMode::Country => stats.apptheme.styles_app.active_border_style,
          _ => stats.apptheme.styles_app.border_style,
        })
        .title(Title::from("Countries").alignment(Alignment::Left))
        .title(Title::from(sort_indicator).alignment(Alignment::Right)),
    )
    .highlight_style(if sel_item.is_blocked {
      stats.apptheme.styles_app.highlight_item_style.bg(stats.apptheme.colors_app.warn_color.color).fg(stats.apptheme.colors_app.text_color.color)
    } else {
      stats.apptheme.styles_app.highlight_item_style
    })
    .highlight_symbol(">> ");

  countrylist
}

pub fn make_region_list<'a>(stats: &Stats) -> List<'a> {
  let av_regions: Vec<ListItem> = stats
    .regions
    .items
    .iter()
    .map(|i| {
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {
        Style::default().fg(stats.apptheme.colors_app.text_color.color).bg(stats.apptheme.colors_app.warn_color.color)
      } else {
        Style::default().fg(stats.apptheme.colors_app.text_color.color)
      });
      ListItem::new(line)
    })
    .collect();
  let mut sel_item = Region::default();
  let sel_idx = stats.regions.state.selected();
  if sel_idx.is_some() && !stats.regions.items.is_empty() {
    sel_item = stats.regions.items[sel_idx.unwrap()].0.clone();
  }
  let sort_indicator = make_sort_state_indicator(&stats.apptheme, stats.regions_sort);
  // Create a List from all list items and highlight the currently selected one
  let regionlist: List<'_> = List::new(av_regions)
    .bg(stats.apptheme.colors_app.background_darkest.color)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(match stats.selection_mode {
          SelectionMode::Region => stats.apptheme.styles_app.active_border_style,
          _ => stats.apptheme.styles_app.border_style,
        })
        .title(Title::from("Regions").alignment(Alignment::Left))
        .title(Title::from(sort_indicator).alignment(Alignment::Right)),
    )
    .highlight_style(if sel_item.is_blocked {
      stats.apptheme.styles_app.highlight_item_style.bg(stats.apptheme.colors_app.warn_color.color).fg(stats.apptheme.colors_app.text_color.color)
    } else {
      stats.apptheme.styles_app.highlight_item_style
    })
    .highlight_symbol(">> ");

  regionlist
}

pub fn make_city_list<'a>(stats: &Stats) -> List<'a> {
  let av_cities: Vec<ListItem> = stats
    .cities
    .items
    .iter()
    .map(|i| {
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {
        Style::default().fg(stats.apptheme.colors_app.text_color.color).bg(stats.apptheme.colors_app.warn_color.color)
      } else {
        Style::default().fg(stats.apptheme.colors_app.text_color.color)
      });
      ListItem::new(line)
    })
    .collect();
  let mut sel_item = City::default();
  let sel_idx = stats.cities.state.selected();
  if sel_idx.is_some() && !stats.cities.items.is_empty() {
    sel_item = stats.cities.items[sel_idx.unwrap()].0.clone();
  }
  let sort_indicator = make_sort_state_indicator(&stats.apptheme, stats.cities_sort);
  // Create a List from all list items and highlight the currently selected one
  let citylist: List<'_> = List::new(av_cities)
    .bg(stats.apptheme.colors_app.background_darkest.color)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(match stats.selection_mode {
          SelectionMode::City => stats.apptheme.styles_app.active_border_style,
          _ => stats.apptheme.styles_app.border_style,
        })
        .title(Title::from("Cities").alignment(Alignment::Left))
        .title(Title::from(sort_indicator).alignment(Alignment::Right)),
    )
    .highlight_style(if sel_item.is_blocked {
      stats.apptheme.styles_app.highlight_item_style.bg(stats.apptheme.colors_app.warn_color.color).fg(stats.apptheme.colors_app.text_color.color)
    } else {
      stats.apptheme.styles_app.highlight_item_style
    })
    .highlight_symbol(">> ");

  citylist
}

pub fn make_isp_list<'a>(stats: &Stats) -> List<'a> {
  let av_isps: Vec<ListItem> = stats
    .isps
    .items
    .iter()
    .map(|i| {
      let is_blocked = i.0.is_blocked;
      let mut line = Line::from(format!("{}", i.0.name.clone()));
      line.patch_style(if is_blocked {
        Style::default().fg(stats.apptheme.colors_app.text_color.color).bg(stats.apptheme.colors_app.warn_color.color)
      } else {
        Style::default().fg(stats.apptheme.colors_app.text_color.color)
      });
      ListItem::new(line)
    })
    .collect();
  let mut sel_item = ISP::default();
  let sel_idx = stats.isps.state.selected();
  if sel_idx.is_some() && !stats.isps.items.is_empty() {
    sel_item = stats.isps.items[sel_idx.unwrap()].0.clone();
  }
  let sort_indicator = make_sort_state_indicator(&stats.apptheme, stats.isps_sort);
  // Create a List from all list items and highlight the currently selected one
  let isplist: List<'_> = List::new(av_isps)
    .bg(stats.apptheme.colors_app.background_darkest.color)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(match stats.selection_mode {
          SelectionMode::ISP => stats.apptheme.styles_app.active_border_style,
          _ => stats.apptheme.styles_app.border_style,
        })
        .title(Title::from("ISPs").alignment(Alignment::Left))
        .title(Title::from(sort_indicator).alignment(Alignment::Right)),
    )
    .highlight_style(if sel_item.is_blocked {
      stats.apptheme.styles_app.highlight_item_style.bg(stats.apptheme.colors_app.warn_color.color).fg(stats.apptheme.colors_app.text_color.color)
    } else {
      stats.apptheme.styles_app.highlight_item_style
    })
    .highlight_symbol(">> ");

  isplist
}

pub fn make_ip_list<'a>(stats: &Stats) -> List<'a> {
  let av_ips: Vec<ListItem> = stats
    .ips
    .items
    .iter()
    .map(|i| {
      let line = Line::from(i.ip.clone());
      ListItem::new(line).style(Style::default().fg(stats.apptheme.colors_app.text_color.color))
    })
    .collect();
  let sel_item = stats.selected_ip.clone();
  let sort_indicator = make_sort_state_indicator(&stats.apptheme, stats.ips_sort);
  // Create a List from all list items and highlight the currently selected one
  let iplist: List<'_> = List::new(av_ips)
    .bg(stats.apptheme.colors_app.background_darkest.color)
    .block(
      Block::default()
        .borders(Borders::ALL)
        .border_style(match stats.selection_mode {
          SelectionMode::IP => stats.apptheme.styles_app.active_border_style,
          _ => stats.apptheme.styles_app.border_style,
        })
        .title(Title::from("IPs").alignment(Alignment::Left))
        .title(Title::from(sort_indicator).alignment(Alignment::Right)),
    )
    .highlight_style(if sel_item.is_banned {
      stats.apptheme.styles_app.highlight_item_style.bg(stats.apptheme.colors_app.warn_color.color).fg(stats.apptheme.colors_app.text_color.color)
    } else {
      stats.apptheme.styles_app.highlight_item_style
    })
    .highlight_symbol(">> ");
  iplist
}

pub fn make_sort_state_indicator<'a>(theme: &Theme, sort_state: SortState) -> Line<'a> {
  let sortstate: (u8, &str, Style) = match sort_state {
    SortState::Alphabetical => (0, "⬆", theme.styles_app.active_border_style),
    SortState::AlphabeticalRev => (0, "⬇", theme.styles_app.active_border_style),
    SortState::NumWarns => (1, "⬆", theme.styles_app.active_border_style),
    SortState::NumWarnsRev => (1, "⬇", theme.styles_app.active_border_style),
    SortState::Blocked => (2, "⬆", theme.styles_app.active_border_style),
    SortState::BlockedRev => (2, "⬇", theme.styles_app.active_border_style),
  };
  let default_text_style = Style::default().fg(theme.colors_app.text_color.color);
  let sort_indicator = Line::from(vec![
    Span::styled("[ ", Style::default().fg(theme.colors_app.text_color.color)),
    if sortstate.0 == 0 {
      Span::styled(format!("ABC {} ", sortstate.1), sortstate.2)
    } else {
      Span::styled("ABC   ", default_text_style)
    },
    Span::styled("|", default_text_style),
    if sortstate.0 == 1 {
      Span::styled(format!(" Warn {} ", sortstate.1), sortstate.2)
    } else {
      Span::styled(" Warn   ", default_text_style)
    },
    Span::styled("|", default_text_style),
    if sortstate.0 == 2 {
      Span::styled(format!(" Block {} ", sortstate.1), sortstate.2)
    } else {
      Span::styled(" Block   ", default_text_style)
    },
    Span::styled(" ]", default_text_style),
  ]);
  sort_indicator
}

// Rendered Overviews // ---------------------------------------------------------------- //

/// Assembles the Lines
pub fn make_overview_paragraph<'a>(
  title: &str,
  theme: &Theme,
  item_name: &str,
  warnings: u32,
  total_warn: u32,
  banned: u32,
  total_banned: u32,
  is_blocked: bool,
) -> Paragraph<'a> {
  let selected_warn_percent_total: f64;
  let selected_ban_percent_total: f64;

  if warnings > 0 {
    selected_warn_percent_total = 100. / (f64::from(total_warn) / f64::from(warnings));
  } else {
    selected_warn_percent_total = 0.;
  }
  if banned > 0 {
    selected_ban_percent_total = 100. / (f64::from(total_banned) / f64::from(banned));
  } else {
    selected_ban_percent_total = 0.;
  }
  let default_text_style = Style::default().fg(theme.colors_app.text_color.color);
  let lines: Vec<Line> = vec![
    Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(theme.colors_app.background_darkest.color))]),
    Line::from(vec![Span::styled(format!(" {}", item_name), Style::default().fg(theme.colors_app.accent_color_a_var.color))]),
    Line::from(vec![if is_blocked {
      Span::styled(format!(" Blocked"), Style::default().fg(theme.colors_app.accent_color_a.color))
    } else {
      Span::styled(format!(" Open   "), Style::default().fg(theme.colors_app.confirm_color.color))
    }]),
    Line::from(vec![Span::styled(format!(" Warnings     : {}", warnings), default_text_style)]),
    Line::from(vec![Span::styled(
      format!(" % total      : {number:.prec$} %", number = selected_warn_percent_total, prec = 2),
      default_text_style,
    )]),
    Line::from(vec![Span::styled(format!(" Total        : {}", total_warn), default_text_style)]),
    Line::from(vec![Span::styled(format!(" Banned       : {}", banned), default_text_style)]),
    Line::from(vec![Span::styled(
      format!(" % total      : {number:.prec$} %", number = selected_ban_percent_total, prec = 2),
      default_text_style,
    )]),
    Line::from(vec![Span::styled(format!(" Total        : {}", total_banned), default_text_style)]),
  ];

  let paragraph = Paragraph::new(lines);
  paragraph.block(Block::default().borders(Borders::ALL).title(format!("{title} Overview")).bg(theme.colors_app.background_darkest.color).fg(theme.colors_app.text_color.color))
  .set_style(Style::new().bg(theme.colors_app.background_darkest.color).fg(theme.colors_app.text_color.color))
}

/// Gather and validates data, wraps make_overview_paragraph()
pub fn make_country_overview(stats: &Stats) -> impl Widget + '_ {
  // get totals
  let mut total_warn: u32 = 0;
  let mut total_banned: u32 = 0;
  let selected_warn: u32;
  let selected_banned: u32;
  let selected_warn_percent_total: f64;
  let selected_ban_percent_total: f64;
  let is_blocked: bool;

  let mut paragraph = Paragraph::new(vec![]);

  let sel_idx = stats.countries.state.selected();
  if sel_idx.is_some() {
    let sel_idx = sel_idx.unwrap();
    for i in 0..stats.countries.items.len() {
      let tuple = stats.countries.items[i].clone();
      total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
      total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));
    }
    let tuple = stats.countries.items[sel_idx].clone();

    selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
    selected_banned = tuple.0.banned.try_into().unwrap_or(0);
    is_blocked = tuple.0.is_blocked;

    paragraph = make_overview_paragraph(
      "Country",
      &stats.apptheme,
      &tuple.0.name,
      selected_warn,
      total_warn,
      selected_banned,
      total_banned,
      is_blocked,
    );
  }
  paragraph.block(Block::default().borders(Borders::ALL).title("Country Stats").bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
  .set_style(Style::new().bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
}

pub fn make_region_overview(stats: &Stats) -> impl Widget + '_ {
  // get totals
  let mut total_warn: u32 = 0;
  let mut total_banned: u32 = 0;
  let selected_warn: u32;
  let selected_banned: u32;
  let selected_warn_percent_total: f64;
  let selected_ban_percent_total: f64;
  let is_blocked: bool;

  let mut paragraph = Paragraph::new(vec![]);

  let sel_idx = stats.regions.state.selected();
  if sel_idx.is_some() {
    let sel_idx = sel_idx.unwrap();
    for i in 0..stats.regions.items.len() {
      let tuple = stats.regions.items[i].clone();
      total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
      total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));
    }
    let tuple = stats.regions.items[sel_idx].clone();
    selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
    selected_banned = tuple.0.banned.try_into().unwrap_or(0);
    is_blocked = tuple.0.is_blocked;

    paragraph = make_overview_paragraph(
      "Region",
      &stats.apptheme,
      &tuple.0.name,
      selected_warn,
      total_warn,
      selected_banned,
      total_banned,
      is_blocked,
    );
  }
  paragraph
    .block(Block::default().borders(Borders::ALL).title("Region Stats").bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
    .set_style(Style::new().bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
}

pub fn make_city_overview(stats: &Stats) -> impl Widget + '_ {
  // get totals
  let mut total_warn: u32 = 0;
  let mut total_banned: u32 = 0;
  let selected_warn: u32;
  let selected_banned: u32;
  let selected_warn_percent_total: f64;
  let selected_ban_percent_total: f64;
  let is_blocked: bool;

  let mut paragraph = Paragraph::new(vec![]);

  let sel_idx = stats.cities.state.selected();
  if sel_idx.is_some() {
    let sel_idx = sel_idx.unwrap();
    for i in 0..stats.cities.items.len() {
      let tuple = stats.cities.items[i].clone();
      total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
      total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));
    }
    let tuple = stats.cities.items[sel_idx].clone();
    selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
    selected_banned = tuple.0.banned.try_into().unwrap_or(0);
    is_blocked = tuple.0.is_blocked;
    paragraph = make_overview_paragraph(
      "City",
      &stats.apptheme,
      &tuple.0.name,
      selected_warn,
      total_warn,
      selected_banned,
      total_banned,
      is_blocked,
    );
  }
  paragraph.block(Block::default().borders(Borders::ALL).title("City Stats").bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
  .set_style(Style::new().bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
}

pub fn make_isp_overview(stats: &Stats) -> impl Widget + '_ {
  // get totals
  let mut total_warn: u32 = 0;
  let mut total_banned: u32 = 0;
  let selected_warn: u32;
  let selected_banned: u32;
  let selected_warn_percent_total: f64;
  let selected_ban_percent_total: f64;
  let is_blocked: bool;

  let mut paragraph = Paragraph::new(vec![]);

  let sel_idx = stats.isps.state.selected();
  if sel_idx.is_some() {
    let sel_idx = sel_idx.unwrap();
    for i in 0..stats.isps.items.len() {
      let tuple = stats.isps.items[i].clone();
      total_banned = total_banned.saturating_add(tuple.0.banned.try_into().unwrap_or(0));
      total_warn = total_warn.saturating_add(tuple.0.warnings.try_into().unwrap_or(0));
    }
    let tuple = stats.isps.items[sel_idx].clone();
    selected_warn = tuple.0.warnings.try_into().unwrap_or(0);
    selected_banned = tuple.0.banned.try_into().unwrap_or(0);
    is_blocked = tuple.0.is_blocked;
    paragraph = make_overview_paragraph(
      "ISP",
      &stats.apptheme,
      &tuple.0.name,
      selected_warn,
      total_warn,
      selected_banned,
      total_banned,
      is_blocked,
    );
  }
  paragraph.block(Block::default().borders(Borders::ALL).title("ISP Stats").bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
  .set_style(Style::new().bg(stats.apptheme.colors_app.background_darkest.color).fg(stats.apptheme.colors_app.text_color.color))
}

pub fn make_ip_overview(theme: &Theme, sel_ip: IP) -> impl Widget + '_ {
  // get totals
  if sel_ip == IP::default() {
    return Paragraph::new(vec![]);
  } //stats.make_ip_overview_old();}

  let selected_ip = sel_ip.clone();

  let ip = selected_ip.ip;
  let warn = selected_ip.warnings;
  let is_banned = selected_ip.is_banned;
  let banned_times = selected_ip.banned_times;
  let country = selected_ip.country;
  let city = selected_ip.city;
  let region = selected_ip.region;
  let isp = selected_ip.isp;

  let default_text_style = Style::default().fg(theme.colors_app.text_color.color);
  let lines: Vec<Line> = vec![
    Line::from(vec![Span::styled(format!(" Selected     :"), Style::default().bg(theme.colors_app.background_darkest.color))]),
    Line::from(vec![Span::styled(format!(" {}", ip), Style::default().fg(theme.colors_app.accent_color_a_var.color))]),
    Line::from(vec![if is_banned {
      Span::styled(format!(" Banned  "), Style::default().fg(theme.colors_app.accent_color_a.color))
    } else {
      Span::styled(format!(" Welcomed"), Style::default().fg(theme.colors_app.confirm_color.color))
    }]),
    Line::from(vec![Span::styled(format!(" Warnings     : {}", warn), default_text_style)]),
    Line::from(vec![Span::styled(format!(" Banned times : {}", banned_times), default_text_style)]),
    Line::from(vec![Span::styled(format!(" {city}, {region}, {country} ",), default_text_style)]),
    Line::from(vec![Span::styled(format!(" {isp} "), default_text_style)]),
  ];

  let paragraph = Paragraph::new(lines);

  paragraph.block(Block::default().borders(Borders::ALL).title("IP Stats").bg(theme.colors_app.background_darkest.color).fg(theme.colors_app.text_color.color))
  .set_style(Style::new().bg(theme.colors_app.background_darkest.color).fg(theme.colors_app.text_color.color))
}

// POPUPS // ---------------------------------------------------------------- //

pub fn popup_help(theme: &Theme) -> impl Widget + '_ {
  // make a layout in center of the screen, outside this function, pass area to this
  let headerstyle = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_text_bright.color);
  let linestyle = Style::default().fg(theme.colors_app.text_color.color);
  let linestyle_alt: Style;
  if theme.is_light {
    linestyle_alt = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_mid.shade(0.5));
  } else {
    linestyle_alt = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_mid.color);
  }
  // make text
  let mut helptext: Vec<Line> = vec![];
  helptext.push(Line::from(Span::styled(format!("Key:          Name          Info"), linestyle)));
  let mut hheader = Line::from(format!("---           General       ---                                                                 -"
  ));
  hheader.patch_style(headerstyle);
  helptext.push(hheader);
  helptext.push(Line::from(Span::styled(format!("Arrowkeys:    Select        Select item in List"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("BackTab:      Switch        Switch selected List up"), linestyle_alt)));
  helptext.push(Line::from(Span::styled(format!("Tab:          Switch        Switch selected List down"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("W|w:          Help          Toggle help"), linestyle_alt)));
  helptext.push(Line::from(Span::styled(format!("R|r:          Refresh       Gets up-to-data for List from db"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("                            (Country auto-fetches all)"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("E|e:          Back          Return to main screen"), linestyle_alt)));
  helptext.push(Line::from(Span::styled(format!("B|b:          Block         Blocks all IPs for selected"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("U|u:          Unblock       Lifts the Block for selected"), linestyle_alt)));
  let mut hheader = Line::from(format!("---           Sorting      ---                                                                 -"
  ));
  hheader.patch_style(headerstyle);
  helptext.push(hheader);  
  helptext.push(Line::from(Span::styled(format!("A|a:          ABC           Sorts selected List by Alpha-Numeric"), linestyle)));
  helptext.push(Line::from(Span::styled(format!("S|s:          Warn          Sorts selected List by number of warnings"), linestyle_alt)));
  helptext.push(Line::from(Span::styled(format!("D|d:          Block         Sorts selected List by blocked / unblocked"), linestyle)));

  let infoblock = Paragraph::new(helptext)
  .set_style(Style::default())
  .block(Block::default()
  .bg(theme.colors_app.background_darkest.color)
  .fg(theme.colors_app.background_text_bright.color)
  .borders(Borders::ALL)
  .border_style(Style::default().fg(theme.colors_app.text_color.color))
  .title("Help"));
  infoblock
}

pub fn popup_un_block_selected(stats: &Stats, is_block: bool) -> impl Widget + '_ {
  let smode = stats.selection_mode;

  let modestr = match smode {
    SelectionMode::Country => "Country",
    SelectionMode::Region => "Region",
    SelectionMode::City => "City",
    SelectionMode::ISP => "ISP",
    SelectionMode::IP => "IP",
  };
  let sel_str = match smode {
    SelectionMode::Country => {
      if stats.countries.items.is_empty() {
        format!("")
      } else {
        let sel_idx = stats.countries.state.selected().unwrap();
        let sel_item = stats.countries.items[sel_idx].0.clone().name;
        sel_item
      }
    },
    SelectionMode::Region => {
      if stats.countries.items.is_empty() {
        format!("")
      } else {
        let sel_idx = stats.regions.state.selected().unwrap();
        let sel_item = stats.regions.items[sel_idx].0.clone().name;
        sel_item
      }
    },
    SelectionMode::City => {
      if stats.countries.items.is_empty() {
        format!("")
      } else {
        let sel_idx = stats.cities.state.selected().unwrap();
        let sel_item = stats.cities.items[sel_idx].0.clone().name;
        sel_item
      }
    },
    SelectionMode::ISP => {
      if stats.countries.items.is_empty() {
        format!("")
      } else {
        let sel_idx = stats.isps.state.selected().unwrap();
        let sel_item = stats.isps.items[sel_idx].0.clone().name;
        sel_item
      }
    },
    SelectionMode::IP => {
      if stats.countries.items.is_empty() {
        format!("")
      } else {
        let sel_idx = stats.ips.state.selected().unwrap();
        let sel_item = stats.ips.items[sel_idx].ip.clone();
        sel_item
      }
    },
  };

  let default_text_style = Style::default().fg(stats.apptheme.colors_app.text_color.color);

  let blockstr = if is_block { "BLOCK" } else { "UNBLOCK" };

  let mut infospan: Span = Span::styled(format!(""), default_text_style);
  if !is_block {
    infospan =
      Span::styled(format!("NOTE: Unblocking does not lead to unbanning any IPs"), default_text_style);
  }

  let mut clearlisttext: Vec<Line> = vec![];
  let clearlistline = Line::from(vec![
    Span::styled(format!("Press "), default_text_style),
    Span::styled(format!("Y | y "), Style::default().fg(stats.apptheme.colors_app.confirm_color.color)),
    Span::styled(format!("to confirm or "), default_text_style),
    Span::styled(format!("N | n "), Style::default().fg(stats.apptheme.colors_app.accent_color_a.color)),
    Span::styled(format!("to cancel."), default_text_style),
  ]);
  //clearlistline.patch_style(stats.apptheme.selected_ip_bg);
  clearlisttext.push(Line::from(vec![infospan]));
  clearlisttext.push(clearlistline);

  let clearlistbox =
    Paragraph::new(clearlisttext).alignment(Alignment::Center)
    .set_style( Style::default().fg(stats.apptheme.colors_app.background_mid.color))
    .block(
      Block::default()
        .borders(Borders::ALL)
        .title(format!("[ Confirm to {} all IPs for {} -> {} ]", blockstr, modestr, sel_str))
        .style(default_text_style.bg(stats.apptheme.colors_app.background_darkest.color))
        .title_alignment(Alignment::Center),
    );
  clearlistbox
}

// CHARTS // ---------------------------------------------------------------- //

pub fn make_bars_for_timestamps<'a>(theme: &Theme, timestamps: Vec<DateTime<FixedOffset>>) -> Vec<Bar<'a>> {
  if timestamps.is_empty() {
    return vec![Bar::default()];
  };
  let mut babars: Vec<Bar> = vec![];
  let mut aday: DateTime<FixedOffset> = timestamps[0];
  let mut num_aday: usize = 0;
  let mut color_switcher: bool = false;
  for stamp in timestamps {
    let day_diff = stamp.day() - aday.day();
    if day_diff > 0 {
      // new bar
      let abar = Bar::default()
        .label(aday.format("%d/%m").to_string().into())
        .value(num_aday as u64)
        .style(if color_switcher {
          color_switcher = false;
          Style::default().fg(theme.colors_app.accent_color_a.color)
        } else {
          color_switcher = true;
          Style::default().fg(theme.colors_app.accent_color_b_mid.color)
        })
        .value_style(Style::default().bg(theme.colors_app.background_brightest.color));
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
      .style(if color_switcher {
        //color_switcher = false;
        Style::default().fg(theme.colors_app.accent_color_a.color)
      } else {
        //color_switcher = true;
        Style::default().fg(theme.colors_app.accent_color_b_mid.color)
      })
      //.value_style(if color_switcher { theme.default_text_style } else { theme.highlight_item_style });
      .value_style(Style::default().bg(theme.colors_app.background_brightest.color));
    babars.push(abar);
  }
  babars
}

pub fn create_barchart<'a>(theme: &Theme, bars: Vec<Bar<'a>>, titlestr: &'a str) -> BarChart<'a> {
  let barchart = BarChart::default()
  .block(Block::default().title(titlestr).borders(Borders::ALL).set_style(theme.styles_app.border_style))
  .bar_width(10)
  .bar_gap(1)
  .group_gap(3)
  .bar_style(Style::new().bg(theme.colors_app.background_brightest.color))
  .value_style(Style::new().fg(theme.colors_app.text_color.color).bold())
  .label_style(Style::new().white())
  //.data(&bars)
  .data(BarGroup::default().bars(&bars))
  .max(100);
  barchart
}
