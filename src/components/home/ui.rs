use super::{themes::Theme, Home, Mode, StyledLine, IPListItem, PointData, IP, DrawMode, IOMode, Animation, Action, HomeAction, get_first_key_by_action};
use crate::{gen_structs::StatefulList, mode::Mode as AppMode};
use ratatui::{prelude::*, widgets::*};

pub fn create_internal_logs<'a>(home: &'a Home) -> List<'a> {
  let intlogs: Vec<ListItem> = home
    .internal_logs      // .items
    .items
    .iter()
    .map(|i| {
        let line = Line::from(i.as_str()); // let mut lines = vec![Line::from(i.0)];
        ListItem::new(line).style(home.apptheme.styles_app.default_style.bg(home.apptheme.colors_app.background_darkest.color)).fg(home.apptheme.colors_app.text_color.shade(-0.5))
    })
    .collect();

  // Create a List from all list items and highlight the currently selected one
  let logslist = List::new(intlogs)
    .bg(home.apptheme.colors_app.background_darkest.color)
    .block(Block::default().borders(Borders::ALL).border_style(home.apptheme.styles_app.border_style).title("INTERNAL LOG"))
    .highlight_style(home.apptheme.styles_app.highlight_item_style)
    .highlight_symbol(">> ");
  logslist
}

// LISTS // ---------------------------------------------------------------- //

pub fn create_io_list<'a>(
  st_st_io: StatefulList<(StyledLine, String, String)>,
  iostreamed_capacity: usize,
  theme: &Theme,
  term_w: usize,
  av_actions: StatefulList<(&'a str, String)>,
  selected_ip: String,
  elapsed_rticks: usize,
) -> List<'a> {
  const ANIMSYMBOLS: [&'static str; 4] = ["|", "/", "―", "\\"];

  let iolines: Vec<ListItem> = st_st_io
    .items
    .iter()
    .map(|i| {
      let mut line: Line = Line::default();
      for word in i.0.words.clone() {
        let cspan = Span::styled(word.0, word.1);
        line.spans.push(cspan);
      }

      let mut bg_style: Style;
      if i.1 == "Journal" {
        if theme.is_light {
          bg_style = Style::default().bg(theme.colors_app.background_darkest.color);
         } else {
          bg_style = Style::default().bg(theme.colors_app.background_darkest.color);
        }
        
      } else {
        bg_style = Style::default().bg(theme.colors_app.background_text_bright.color);
      }

      if i.2 == selected_ip {
        if theme.is_light {
          bg_style = Style::default().bg(theme.colors_app.background_text_bright.color);
        } else {
          bg_style = Style::default().bg(theme.colors_app.background_text_dark.color);
        }
        
      }

      let line_w = line.width();
      if line_w < term_w {
        // fill line with whitespaces
        let dif = term_w - line_w;
        let cspan = Span::styled(str::repeat(" ", dif), Style::default().fg(theme.colors_app.text_color.color));
        line.spans.push(cspan);
      }
      line.patch_style(bg_style);
      ListItem::new(line)
    })
    .collect();

  let mut ioactive: u8 = 0;
  if av_actions.items[2].1 == "active" || av_actions.items[3].1 == "active" {
    if av_actions.items[2].1 == "active" && av_actions.items[3].1 == "active" {
      ioactive = 2;
    } else {
      ioactive = 1;
    }
  }
  let default_text_style = Style::default().fg(theme.colors_app.text_color.color);
  let iolist_title = Line::from(vec![
    Span::styled(" I/O Stream [ ", default_text_style),
    Span::styled(
      ANIMSYMBOLS[elapsed_rticks],
      match ioactive {
        0 => Style::default().fg(theme.colors_app.warn_color.color),
        1 => Style::default().fg(theme.colors_app.error_color.color),
        2 => Style::default().fg(theme.colors_app.accent_color_b_mid.color),
        _ => Style::default().fg(theme.colors_app.warn_color.color),
      },
    ),
    Span::styled(" ] ", default_text_style),
  ]);

  let iolist_selected_idx = st_st_io.state.selected();
  let selected_symb = if iolist_selected_idx.is_some() {
    let selnum = iolist_selected_idx.unwrap() + 1;
    selnum.to_string()
  } else {
    String::from("-")
  };
  let ciolist_len = st_st_io.items.len();
  let list_capacity_diff = iostreamed_capacity - ciolist_len;

  let capacity_color = if list_capacity_diff < 10 {
    if list_capacity_diff == 0 {
      theme.colors_app.accent_color_a.color
    } else {
      theme.colors_app.accent_color_b_mid.color
    }
  } else {
    theme.colors_app.text_color.color
  };

  let iolist_capacity_display = Line::from(vec![
    Span::styled(format!("[ "), default_text_style),
    Span::styled(format!("{}", selected_symb), Style::default().fg(theme.colors_app.accent_color_b_mid.color)), // selected
    Span::styled(format!(" : "), default_text_style),                                    // separator
    Span::styled(format!("{}", ciolist_len), Style::default().fg(capacity_color)),             // current num
    Span::styled(format!(" / ",), default_text_style),                                   // separator
    Span::styled(format!("{}", iostreamed_capacity), Style::default().fg(capacity_color)),     // capacity
    Span::styled(format!(" ]"), default_text_style),
  ]);

  // Create a List from all list items and highlight the currently selected one
  let iolist = List::new( iolines) //home.styledio.clone()
            .block(Block::default()
              .bg(theme.colors_app.background_darkest.color)
              .borders(Borders::ALL)
              .border_style(default_text_style)
              .title(block::Title::from(iolist_title).alignment(Alignment::Left))
              .title(block::Title::from(iolist_capacity_display).alignment(Alignment::Right))
            )
            //.highlight_style(theme.highlight_item_style)
            .highlight_symbol(">> ");
  iolist
}

pub fn create_ip_list(iplist: StatefulList<IPListItem>, theme: &Theme, mode: Mode, last_mode:Mode) -> List<'_> {
  let ips: Vec<ListItem> = iplist      // .items
  .items
  .iter()
  .map(|i| {
      let mut lines = vec![Line::from(format!("{}   {}", i.IP.ip, i.username))]; // let mut lines = vec![Line::from(i.0)];
      let mut symb = "X";
      if i.IP.is_banned  {
        symb = "✓";
      }
      lines.push(
        format!("{} - {}, {}", symb, i.IP.city, i.IP.country)
            .italic()
            .into(),
      );
      ListItem::new(lines).style(Style::default().fg(theme.colors_app.text_color.color))
  })
  .collect();

  // Create a List from all list items and highlight the currently selected one
  let iplist = List::new(ips)
      .bg(theme.colors_app.background_darkest.color)
      .block(Block::default()
      .borders(Borders::ALL)
      .border_style( 
        match mode {
          Mode::Normal => {theme.styles_app.active_border_style},
          _ => {  let mut style = theme.styles_app.border_style;
            if last_mode == Mode::Normal {style = theme.styles_app.active_border_style}
            style},
        })
      .title("Last IPs"))
      .highlight_style(theme.styles_app.highlight_item_style)
      .highlight_symbol(">> ");
    iplist
}

pub fn create_action_list<'a>(available_actions:StatefulList<(&'a str, String)>, theme: &Theme, mode: Mode, last_mode:Mode , selected_ip: String) -> List<'a> {
  let av_actions: Vec<ListItem> = available_actions
  .items
  .iter()
  .map(|i| {
      let mut lines = vec![Line::from(i.0)];
      if i.0 == "Ban" || i.0 == "Unban"
      {
        lines.push(
          format!("   {}", selected_ip)
              .italic()
              .into(),
        );          
      }
      else if i.0 == "monitor-fail2ban" || i.0 == "monitor-journalctl"{
        let mut symb = "X";
        if i.1 == String::from("active") {
          symb = "✓";
        }
        lines.push(
            format!("   {} - {}", symb, i.1)
            .italic()
            .into(),
        );            
      }
      else {
        lines.push(
          format!("   Hotkey: {}", i.1)
          .italic()
          .into(),
      ); 
      }


      ListItem::new(lines).style(Style::new().bg(theme.colors_app.background_darkest.color).fg(theme.colors_app.text_color.color))
  })
  .collect();

  // Create a List from all list items and highlight the currently selected one
  let actionlist = List::new(av_actions)
      .bg(theme.colors_app.background_darkest.color)
      .block(Block::default()
      .borders(Borders::ALL)
      .border_style( 
        match mode {
          Mode::TakeAction => {theme.styles_app.active_border_style},
          _ => {  let mut style = theme.styles_app.border_style;
            if last_mode == Mode::TakeAction {style = theme.styles_app.active_border_style}
            style},
        })
      .title("Actions"))
      .highlight_style(theme.styles_app.highlight_item_style)
      .highlight_symbol(">> ");
    actionlist
}

// POPUPS // ---------------------------------------------------------------- //

pub fn create_help_popup<'a>(home: &'a Home) -> impl Widget + 'a {
  // make a layout in center of the screen, outside this function, pass area to this  
  let active_drawmode = match home.drawmode {
    DrawMode::All => {"All   "},
    DrawMode::Decaying => {"Decay "},
    DrawMode::Sticky => {"Sticky"},
  };
  let active_iomode = match home.iomode {
    IOMode::Follow => {"Follow"},
    IOMode::Static => {"Static"},
  };


  // find relevant keys,
  let keymap = home.config.keybindings.0.get(&AppMode::Home).unwrap();
  let key_help = get_first_key_by_action(keymap, Action::Help);
  let key_query = get_first_key_by_action(keymap, Action::Home(HomeAction::Query));
  let key_stats = get_first_key_by_action(keymap, Action::Home(HomeAction::EnterStats));
  let key_logs = get_first_key_by_action(keymap, Action::Home(HomeAction::Logs));
  let key_map = get_first_key_by_action(keymap, Action::Home(HomeAction::Map));
  let key_clear = get_first_key_by_action(keymap, Action::Home(HomeAction::Clear));
  let key_ban = get_first_key_by_action(keymap, Action::Home(HomeAction::Ban));
  let key_unban = get_first_key_by_action(keymap, Action::Home(HomeAction::Unban));

  let key_follow = get_first_key_by_action(keymap, Action::Home(HomeAction::Follow));
  let key_static = get_first_key_by_action(keymap, Action::Home(HomeAction::Static));

  let key_draw_all = get_first_key_by_action(keymap, Action::Home(HomeAction::DrawAll));
  let key_draw_sticky = get_first_key_by_action(keymap, Action::Home(HomeAction::DrawSticky));
  let key_draw_decay = get_first_key_by_action(keymap, Action::Home(HomeAction::DrawDecay));

  let key_first = get_first_key_by_action(keymap, Action::Home(HomeAction::First));
  let key_prev = get_first_key_by_action(keymap, Action::Home(HomeAction::Previous));
  let key_next = get_first_key_by_action(keymap, Action::Home(HomeAction::Next));
  let key_last = get_first_key_by_action(keymap, Action::Home(HomeAction::Last));
  let key_unsel = get_first_key_by_action(keymap, Action::Home(HomeAction::Unselect));
  let key_capac = get_first_key_by_action(keymap, Action::Home(HomeAction::SetCapacity));


  let headerstyle = Style::default().fg(home.apptheme.colors_app.text_color.color).bg(home.apptheme.colors_app.background_text_bright.color);
  let linestyle = Style::default().fg(home.apptheme.colors_app.text_color.color);
  let linestyle_alt: Style;
  if home.apptheme.is_light {
    linestyle_alt = Style::default().fg(home.apptheme.colors_app.text_color.color).bg(home.apptheme.colors_app.background_mid.shade(0.5));
  } else {
    linestyle_alt = Style::default().fg(home.apptheme.colors_app.text_color.color).bg(home.apptheme.colors_app.background_mid.color);
  }
  
  // make text
  let mut helptext: Vec<Line> = vec![];
  helptext.push(                Line::from(Span::styled(format!("Key:          Name          Info"), linestyle)));
  let mut hheader =   Line::from(                     format!("---           General       ---                                                                 -"));
  hheader.patch_style(headerstyle);
  helptext.push(hheader);    
  helptext.push(                Line::from(Span::styled(format!("Arrowkeys:    Select        Select item in IPs or Actions dependent on mode"), linestyle)));
  helptext.push(                Line::from(Span::styled(format!("Tab:          Mode          Switch Mode between IP-List & Actions"), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Help          Toggle help", key_help), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Query         Toggle query input for IP data from db", key_query), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Ban           Ban entered IP", key_ban), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Unban         Unban entered IP", key_unban), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Stats         Switch to Stats-Screen", key_stats), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Logs          Maximizes Logs", key_logs), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Map           Maximizes Map", key_map), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Clear         Clears IP and I/O Lists", key_clear), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!("Enter:        Execute       Context dependent selection or execution"), linestyle)));
  let mut hheader =   Line::from(                     format!("---           Drawmode      ---                                           {}                -", active_drawmode)); // for more spaces bc inserted string has six characters
  hheader.patch_style(headerstyle);
  helptext.push(hheader);
  helptext.push(                Line::from(Span::styled(format!(" {} :          All           Draws all connections all the time", key_draw_all), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Sticky        Draws only the selection connection", key_draw_sticky), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Decay         Draws each connection for 10 seconds", key_draw_decay), linestyle)));
  let mut ioheader =  Line::from(                     format!("---           I/O Stream    ---                                 Capacity: {}                -", home.iostreamed_capacity));
  ioheader.patch_style(headerstyle);
  helptext.push(ioheader);
  helptext.push(                Line::from(Span::styled(format!(" {} :          First         Select oldest line in I/O Streamed", key_first), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Previous      Select previous line in I/O Streamed", key_prev), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Next          Select next line in I/O Streamed", key_next), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Last          Select latest line in I/O Streamed", key_last), linestyle_alt)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Unselect      Reset line selection in I/O Streamed", key_unsel), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Set Capacity  Input a new capacity for I/O Streamed", key_capac), linestyle_alt)));
  let mut hheader =   Line::from(                     format!("---           IO-Mode       ---                                           {}                -", active_iomode)); // four more spaces bc inserted string has six characters
  hheader.patch_style(headerstyle);
  helptext.push(hheader);
  helptext.push(                Line::from(Span::styled(format!(" {} :          Follow        Auto-selects the last received IP", key_follow), linestyle)));
  helptext.push(                Line::from(Span::styled(format!(" {} :          Static        Selection stays where you left it", key_static), linestyle_alt)));   

  let infoblock = Paragraph::new(helptext)
  .set_style(Style::default())
  .block(Block::default()
  .bg(home.apptheme.colors_app.background_darkest.color)
  .fg(home.apptheme.colors_app.background_text_bright.color)
  .borders(Borders::ALL)
  .border_style(Style::default().fg(home.apptheme.colors_app.text_color.color))
  .title("Help"));
  infoblock

}

pub fn create_query_popup<'a>(home: &'a Home)-> impl Widget + 'a {

  let querycursor = home.anim_querycursor.state.selected().unwrap();
  let querycursor = home.anim_querycursor.keyframes[querycursor];

  let mut querytext: Vec<Line> = vec![];
  let queryline =   Line::from(vec![
    Span::styled(format!("Query: {}", home.querystring), Style::default().bg(home.apptheme.colors_app.background_darkest.color).fg(home.apptheme.colors_app.text_color.color)) , 
    Span::styled(querycursor, Style::default().bg(home.apptheme.colors_app.background_brightest.color).fg(home.apptheme.colors_app.text_color.color))
    ]);
  //queryline.patch_style(home.apptheme.selected_ip_bg);
  querytext.push(queryline);

  let mut queryerror =   Line::from(format!("Status: {}", home.queryerror));
  queryerror.patch_style(Style::default().bg(home.apptheme.colors_app.background_mid.color).fg(home.apptheme.colors_app.text_color.color));
  querytext.push(queryerror);

  let querybox = Paragraph::new(querytext)
  .set_style(Style::default())
  .block(Block::default()
  .bg(home.apptheme.colors_app.background_darkest.color)
  .borders(Borders::ALL)
  .title("Query"));
  querybox

}

pub fn create_clearlist_popup(theme: &Theme)  -> impl Widget + '_  {

  let mut clearlisttext: Vec<Line> = vec![];
  let clearlistline =   Line::from(vec![
    Span::styled(format!("Press "), Style::default().fg(theme.colors_app.text_color.color)), 
    Span::styled(format!("Y | y "), Style::default().fg(theme.colors_app.confirm_color.color)),
    Span::styled(format!("to confirm or "), Style::default().fg(theme.colors_app.text_color.color)),
    Span::styled(format!("N | n "), Style::default().fg(theme.colors_app.accent_color_a.color)),
    Span::styled(format!("to cancel."), Style::default().fg(theme.colors_app.text_color.color)),
    ]);
  //clearlistline.patch_style(theme.selected_ip_bg);
  clearlisttext.push(clearlistline);

  let clearlistbox = Paragraph::new(clearlisttext)
  .set_style(Style::new().fg(theme.colors_app.text_color.color))
  .block(Block::default()
  .bg(theme.colors_app.background_darkest.color)
  .borders(Borders::ALL)
  .border_style(theme.styles_app.border_style)
  .title("Confirm to clear list"));
  clearlistbox
}

pub fn popup_set_io_capacity<'a>(anim_querycursor: Animation<&'a str>, theme: &Theme, capacity_input: String) -> impl Widget + 'a {

  let capacitycursor = anim_querycursor.state.selected().unwrap();
  let capacitycursor = anim_querycursor.keyframes[capacitycursor];

  let mut capacitytext: Vec<Line> = vec![];
  let capacityline =   Line::from(vec![
    Span::styled(format!("New Capacity: {}", capacity_input), Style::new().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_text_dark.color)) , 
    Span::styled(capacitycursor, Style::new().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_text_bright.color))
    ]);
  //capacityline.patch_style(theme.selected_ip_bg);
  capacitytext.push(capacityline);

  let capacitybox = Paragraph::new(capacitytext)
  .set_style(Style::default().fg(theme.colors_app.text_color.color))
  .block(Block::default()
  .bg(theme.colors_app.background_darkest.color)
  .borders(Borders::ALL)
  .border_style(theme.styles_app.border_style)
  .title("Set I/O Stream capacity"));
  capacitybox
}