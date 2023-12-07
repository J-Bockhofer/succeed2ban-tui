use ratatui::prelude::Style;
use super::{Home, StyledLine, IP, PointData, IPListItem, IOMode, Action, UnboundedSender, Result};

/// Styles the incoming lines from either journalctl or fail2ban log.
// Fail2Ban may send a String that contains multiple lines which are delimited by "++++".
// Styled lines (saved in home.stored_styled_iostreamed) contain colored substrings / words.
pub fn style_incoming_message(home: &mut Home, msg: String) {
  let mut dbg: String;

  let mut last_io = String::from("Journal");
  home.last_username = String::from("");

  if msg.contains("++++") {
    // message is from Fail2Ban
    last_io = String::from("Fail2Ban");
  }
  let collected: Vec<&str> = msg.split("++++").collect(); // new line delimiter in received lines, if more than one got added simultaneously
  home.debug_me = collected.clone().len().to_string();
  //home.debug_me = collected.clone().len().to_string();

  for tmp_line in collected {
    if tmp_line.is_empty() {
      continue;
    }
    let mut thisline: StyledLine = StyledLine::default();
    // do word_map matching first then regex match splitting
    // look for ip quickly to send it out to the list
    let results: Vec<&str> = home.apptheme.ipregex
    .captures_iter(tmp_line)
    .filter_map(|capture| capture.get(1).map(|m| m.as_str()))
    .collect();
    let mut cip: &str = "";
    if !results.is_empty() {
      cip = results[0];
    }


    let words: Vec<&str> = tmp_line.split(" ").collect();
    let mut held_unstyled_words: Vec<&str> = vec![];
    let mut last_word: &str= "";

    for word in words.clone(){
      let mut word_style = home.apptheme.word_style_map.get_style_or_default(word.to_string()); // Detector for constant word
      if word_style == Style::default() {
        // try regex styling on word
        word_style = home.apptheme.regex_style_map.get_style_or_default(word.to_string()); // Detector for regex
      } 
      if last_word == "user" {
        word_style = home.apptheme.username_style;
        home.last_username = word.to_string();
      }
      

      if word_style == Style::default() {
        // If no detector has returned any styling
        held_unstyled_words.push(word);
      }
      else {
        // word is styled
        // if there are any held words push them with default style and reset held words
        if held_unstyled_words.len() > 0 {

          thisline.words.push((format!(" {}", held_unstyled_words.join(" ")), home.apptheme.default_text_style));
          held_unstyled_words = vec![];
        }
        // push styled word with space in front - TODO word is in first position
        thisline.words.push((format!(" {}", word.to_string()), word_style));

      }
      last_word = word;

      // terminate
      if &word == words.last().unwrap() {
        thisline.words.push((format!(" {}",held_unstyled_words.join(" ")), home.apptheme.default_text_style));
      }
      

    }

    //home.stored_styled_lines.push(thisline);
    home.stored_styled_iostreamed.items.push((thisline, last_io.clone(), cip.to_string()));
    home.stored_styled_iostreamed.trim_to_length(home.iostreamed_capacity);

  }// end per line

}

/// Parses received IP geodata, calculates direction to home, passes to style message
pub fn parse_passed_geo(home: &mut Home, x: IP, y: String, z: bool) -> Result<()> {
  
  style_incoming_message(home, y.clone());

  let cip = x.ip.clone();

  let cipvec = home.iplist.items.clone();

  if !cipvec.iter().any(|i| i.IP.ip==cip) {
    // if cip isnt in vector yet
    let lat = x.lat.clone().parse::<f64>().unwrap();
    let lon = x.lon.clone().parse::<f64>().unwrap();
    let dir_lat = home.home_lat - lat;
    let dir_lon = home.home_lon - lon;

    home.last_lat = lat.clone();
    home.last_lon = lon.clone();

    home.last_direction = (dir_lon, dir_lat);

    let pointdata = PointData::new(cip.clone(), lon, lat, dir_lon, dir_lat);

    let iplistitem = IPListItem::new(x.clone(), home.last_username.clone(), pointdata);

    //home.iplist.items.push((cip.clone(), x.clone(), home.last_username.clone()));
    home.iplist.items.push(iplistitem);
    home.iplist.trim_to_length(home.iplist_capacity); // change to const

    if home.iomode == IOMode::Follow {
      if home.iplist.items.len() > 1 {
        home.iplist.state.select(Option::Some(home.iplist.items.len()-1));
      }
      else {
        home.iplist.state.select(Option::Some(0));
      }

      home.selected_ip = cip;
    }

    
  } 
  else {
    // ip is already in vector, need to select it again if IOmode is follow
    if home.iomode == IOMode::Follow {
      for i in 0..home.iplist.items.len() {
        let item = &home.iplist.items[i];
        if item.IP.ip == cip {
          home.iplist.state.select(Some(i));
          home.iplist.items[i].pointdata.refresh();
          home.selected_ip = cip;
          break;
        }
      }
    }
  }
  home.command_tx.clone().unwrap().send(Action::Render)?;
  Ok(())
}

