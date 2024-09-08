use ratatui::{style::{Style, Styled, Stylize}, text::{Line, Span}, widgets::{Block, Borders, Paragraph, Widget}};



const COLUMN_GAP: usize = 2;

#[derive(Debug, Clone, Copy, Default)]
pub struct MaxLengths {
    key: usize,
    name: usize,
    info: usize,
    state_descriptor: usize,
    state: usize,
}

impl MaxLengths {
    pub fn compare_and_overwrite(&mut self, rhs: &MaxLengths) {
        self.key = if self.key > rhs.key {self.key} else {rhs.key};
        self.name = if self.name > rhs.name {self.name} else {rhs.name}; 
        self.info = if self.info > rhs.info {self.info} else {rhs.info}; 
        self.state_descriptor = if self.state_descriptor > rhs.state_descriptor {self.state_descriptor} else {rhs.state_descriptor};
        self.state = if self.state > rhs.state {self.state} else {rhs.state}; 
    }

    pub fn get_line_length(&self) -> usize {
        self.key + 1 + COLUMN_GAP + self.name + COLUMN_GAP + self.info + self.state + COLUMN_GAP*2
    }
}

#[derive(Debug, Clone)]
pub struct HelpStyles {
    pub header: Style,
    pub opt: Style,
    pub opt_alt: Style,
}

#[derive(Debug, Clone)]
pub struct HelpOptions {
    pub categories: Vec<HelpOptCategory>,
    pub max_lengths: MaxLengths,
}

impl HelpOptions {
    pub fn from_categories(categories: Vec<HelpOptCategory>) -> Self {
        let mut max_lengths = MaxLengths::default();
        for cat in &categories {
            max_lengths.compare_and_overwrite(&cat.get_max_lengths());
        }
        HelpOptions { categories, max_lengths }
    }
    pub fn get_num_lines(&self) -> usize {
        let mut count: usize = 1; // one for header
        for cat in &self.categories {
            count += cat.num_lines();
        }
        count
    }

    pub fn header_line<'a>(&self, linestyle: Style) -> Line<'a> {
        make_header_line(linestyle, &self.max_lengths)
    }

    pub fn make_lines<'a>(&self, styles: HelpStyles) -> Vec<Line<'a>> {
        let mut lines: Vec<Line> = vec![self.header_line(styles.opt)];
        let mut use_alt: bool = false;
        for cat in &self.categories {
            // category line
            lines.push(cat.header_line(styles.header, &self.max_lengths));
            // option lines
            for opt in &cat.opts {
                let linestyle = if use_alt {styles.opt_alt} else {styles.opt};
                lines.push(opt.to_line(linestyle, &self.max_lengths));
                use_alt = !use_alt;
            }
        }
        lines
    }
}

#[derive(Debug, Clone, Default)]
pub struct HelpOptCategory {
    pub name: String,
    pub name_len: usize,
    pub state_descriptor: String,
    pub state_descriptor_len: usize,
    pub state: String,
    pub state_len: usize,
    pub opts: Vec<HelpOpt>,
}

impl HelpOptCategory {
    pub fn with_name<T: ToString>(mut self, name: T) -> Self {
        self.name = name.to_string();
        self.name_len = self.name.len();
        self
    }
    pub fn with_state_descriptor<T: ToString>(mut self, descriptor: T) -> Self {
        self.state_descriptor = descriptor.to_string();
        self.state_descriptor_len = self.state_descriptor.len();
        self
    }
    pub fn with_state<T: ToString>(mut self, state: T) -> Self {
        self.state = state.to_string();
        self.state_len = self.state.len();
        self
    }
    pub fn with_opts(mut self, opts: Vec<HelpOpt>) -> Self {
        self.opts = opts;
        self
    }
    pub fn get_max_lengths(&self) -> MaxLengths {
        let mut max_lengths = MaxLengths::default();
        max_lengths.name = self.name_len;
        max_lengths.state_descriptor = self.state_descriptor_len;
        max_lengths.state = self.state_len;

        for opt in &self.opts {
            if opt.name_len > max_lengths.name {max_lengths.name = opt.name_len;}
            if opt.key_len > max_lengths.key {max_lengths.key = opt.key_len;}
            if opt.info_len > max_lengths.info {max_lengths.info = opt.info_len;}
        }
        max_lengths
    }

    pub fn num_lines(&self) -> usize {
        self.opts.len() + 1
    }

    pub fn header_line<'a>(&self, linestyle: Style, max_lengths: &MaxLengths) -> Line<'a>{
        make_category_line(self, linestyle, max_lengths)
    }
}

#[derive(Debug, Clone)]
pub struct HelpOpt {
  pub key: String,
  pub key_len: usize,
  pub name: String,
  pub name_len: usize,
  pub info: String,
  pub info_len: usize,
}

use std::string::ToString;

impl HelpOpt{
    pub fn new_opt<T: ToString>(key: T, name: T, info: T) -> Self {

        let key = key.to_string();
        let key_len = key.len();
        let name = name.to_string();
        let name_len = name.len();
        let info = info.to_string();
        let info_len = info.len();

        return HelpOpt { 
            key, 
            key_len, 
            name, 
            name_len, 
            info, 
            info_len }
    }

    pub fn to_line<'a>(&self, linestyle: Style, max_lengths: &MaxLengths) -> Line<'a> {
        make_opt_line(self, linestyle, max_lengths)
    }
}

// calculate overall width and height
// longest line & num lines
// max key+1 + gap + name + gap + info || state + gap


use crate::themes::Theme;

use super::{pad_to_length, leftpad_to_length};

pub fn make_opt_line<'a>(opt: &HelpOpt, linestyle: Style, max_lengths: &MaxLengths) -> Line<'a> {
     // "key:{gap}name{gap}info{gap}state{gap}"
    let mut linestr = String::new();
    // key
    let keystr = if opt.key_len == 1 {format!(" {} :", opt.key)} else {format!("{}:", opt.key)};
    linestr.push_str(&pad_to_length(&keystr, max_lengths.key + 1 + COLUMN_GAP));

    // name
    linestr.push_str(&pad_to_length(&opt.name, max_lengths.name + COLUMN_GAP));

    // info
    linestr.push_str(&pad_to_length(&opt.info, max_lengths.info + COLUMN_GAP));

    // pad to max length
    linestr = pad_to_length(&linestr, max_lengths.get_line_length());

    return Line::from(Span::styled(linestr, linestyle))
}

pub fn make_category_line<'a>(cat: &HelpOptCategory, linestyle: Style, max_lengths: &MaxLengths) -> Line<'a> {
    // format!("---           Drawmode      ---                                           {}                         -", active_drawmode));
    let linelen = max_lengths.get_line_length();
    let keyspacer = pad_to_length("---", max_lengths.key +1 + COLUMN_GAP);
    let name = pad_to_length(&cat.name, max_lengths.name + COLUMN_GAP);
    let mut linestr = String::new();
    linestr.push_str(&keyspacer);
    linestr.push_str(&name);
    // have to find the spacing between placeholder "---" and state descriptor
    let state_pos = linelen - COLUMN_GAP*2 - max_lengths.state;
    let descriptor_pos = if cat.state_descriptor_len >= 1 {state_pos - 2 - max_lengths.state_descriptor} else {state_pos};
    let infospacer_pos = linestr.len() + 3; 
    let infospacer = pad_to_length("---", descriptor_pos - infospacer_pos);
    linestr.push_str(&infospacer);
    // build descriptor
    if cat.state_descriptor_len >= 1 {
        let state_desc =  leftpad_to_length(&format!("{}: ", cat.state_descriptor), max_lengths.state_descriptor + 2);
        linestr.push_str(&state_desc);
    }
    // attach state
    let statespacer = pad_to_length(&cat.state, max_lengths.state);
    linestr.push_str(&statespacer);

    linestr = pad_to_length(&linestr, linelen);

    return Line::from(Span::styled(linestr, linestyle))
}

pub fn make_header_line<'a>(linestyle: Style, max_lengths: &MaxLengths) -> Line<'a> {
    // "key:{gap}name{gap}info{gap}state{gap}"
   let mut linestr = String::new();
   // key
   let keystr = "Key:";
   linestr.push_str(&pad_to_length(keystr, max_lengths.key + 1 + COLUMN_GAP));

   // name
   linestr.push_str(&pad_to_length("Name", max_lengths.name + COLUMN_GAP));

   // info
   linestr.push_str(&pad_to_length("Info", max_lengths.info + COLUMN_GAP));

   // pad to max length
   linestr = pad_to_length(&linestr, max_lengths.get_line_length());

   return Line::from(Span::styled(linestr, linestyle))
}

pub fn help_widget(theme: &Theme, help: HelpOptions) -> impl Widget + '_ {

    let headerstyle = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_text_bright.color);
    let linestyle = Style::default().fg(theme.colors_app.text_color.color);
    let linestyle_alt: Style;
    if theme.is_light {
      linestyle_alt = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_mid.shade(0.5));
    } else {
      linestyle_alt = Style::default().fg(theme.colors_app.text_color.color).bg(theme.colors_app.background_mid.color);
    }
    
    let styles = HelpStyles{
      header: headerstyle,
      opt: linestyle,
      opt_alt: linestyle_alt,
    };
  
    let helptext = help.make_lines(styles);
  
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