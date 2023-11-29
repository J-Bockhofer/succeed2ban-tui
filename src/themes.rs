
use color_eyre::owo_colors::OwoColorize;
use ratatui::{style::Style, prelude::Color, prelude::Modifier};
use regex::Regex;

use std::iter::Map;
//use std::collections::HashMap;

pub mod colors;

#[derive(Default)]
pub struct WordStylePair {
    pub word: String,
    pub style: Style,
}

impl WordStylePair {
    pub fn new(word: String, style: Style) -> Self {
        WordStylePair { word, style }
    }
}



pub struct RegexStylePair {
    pub word_regex: Regex,
    pub style: Style,
}
impl RegexStylePair {
    pub fn new(word_regex: Regex, style: Style) -> Self {
        RegexStylePair { word_regex, style }
    }
}


// match word {
//  
//      span = word + wordmap.get_style_or_default(word)
//  
//}

// Word style map should be used like this
// split incoming line = String into words
// .map the word_style_tuples 
// in the closure {} iterate through
// 
//}
#[derive(Default)]
pub struct WordStyleMap {
    pub word_styles: Vec<WordStylePair>,
}

impl WordStyleMap {
    pub fn new(word_styles: Vec<WordStylePair>) -> Self {
        WordStyleMap { word_styles}
    }
    pub fn word_in_map(&self, word:String) -> bool {
        let mut res = false;
        for item in self.word_styles.iter() {
            if word == item.word {
                res = true;
                return res;
            }
        }
        res
    }

    /// return the style for a given word if found in WordStyleMap
    pub fn get_style_or_default(&self, word:String) -> Style {
        
        for item in self.word_styles.iter() {
            if word == item.word {
                return item.style;
            }
        }
        Style::default()
    }

}

pub struct RegexStyleMap {
    pub regex_styles: Vec<RegexStylePair>,
}

impl RegexStyleMap {
    pub fn new(regex_styles: Vec<RegexStylePair>) -> Self {
        RegexStyleMap { regex_styles}
    }
    pub fn word_in_map(&self, word:String) -> bool {
        for item in self.regex_styles.iter() {

            if item.word_regex.is_match(word.as_str()) {
                return true;
            }
        }
        false
    }

    /// return the style for a given word if found in RegexStyleMap
    pub fn get_style_or_default(&self, word:String) -> Style {
        
        for item in self.regex_styles.iter() {

            if item.word_regex.is_match(word.as_str()) {
                return item.style;
            }
        }
        Style::default()
    }

}

#[derive(Default)]
pub struct ThemeColors {
    pub default_background: Color,
    pub default_map_color: Color,

    pub accent_blue: Color,
    pub accent_orange: Color,
    pub accent_lorange: Color,
    pub accent_dblue: Color,
    pub accent_wred: Color,
    pub accent_lpink: Color,
    pub accent_lime: Color,

    pub lblack: Color,

}



impl ThemeColors {
    pub fn new() -> Self {
        ThemeColors::default()
    }
}

pub struct Theme {
    // Text Styling
    pub word_style_map: WordStyleMap,
    pub regex_style_map: RegexStyleMap,
    pub default_text_style: Style,
    pub username_style: Style,
    pub journal_bg: Style,
    pub fail2ban_bg: Style,
    // UI Styling
    pub default_background: Style,
    pub border_style: Style,
    pub active_border_style: Style,
    pub highlight_item_style: Style,
    // Const Colors
    pub colors: ThemeColors,


} 

impl Theme {
    pub fn new(word_style_map: WordStyleMap, 
        regex_style_map: RegexStyleMap, 
        default_text_style: Style, 
        username_style:Style,
        default_background: Style,
        border_style: Style,
        active_border_style: Style,
        journal_bg: Style,
        fail2ban_bg: Style,
        highlight_item_style: Style, 
        colors: ThemeColors,
        ) -> Self 

        {
        Theme { word_style_map, regex_style_map, default_text_style, username_style, default_background, border_style, active_border_style, 
            journal_bg,
            fail2ban_bg,
            highlight_item_style,
            colors,
        }
    }
}

impl Default for Theme {
    fn default() -> Self {
        //Theme::new()
        Theme { word_style_map: WordStyleMap{ word_styles: vec![
                    WordStylePair::new(String::from("Found"), Style::default().fg(colors::ACCENT_DBLUE)),
                    WordStylePair::new(String::from("Ban"), Style::default().fg(colors::ACCENT_ORANGE)),
                    WordStylePair::new(String::from("INFO"), Style::default().fg(colors::ACCENT_DBLUE)),
                    WordStylePair::new(String::from("WARNING"), Style::default().fg(colors::ACCENT_LPINK)),
                    WordStylePair::new(String::from("NOTICE"), Style::default().fg(colors::ACCENT_LIME)),
                    WordStylePair::new(String::from("user"), Style::default().fg(colors::ACCENT_BLUE)),
                    WordStylePair::new(String::from("fatal:"), Style::default().fg(colors::ACCENT_ORANGE)),
                    //WordStylePair::new(String::from("[sshd]"), Style::default().fg(Color::Rgb(138, 43, 226))),
                ]
            }, 
            regex_style_map: RegexStyleMap{ regex_styles: vec![
                RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // IP v4
                RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // Timestamp HH:MM:SS
                RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // Timestamp HH:MM:SS
            ]},
            default_text_style: Style::default().fg(Color::White),
            username_style: Style::default().fg(colors::ACCENT_ORANGE),
            default_background: Style::default().bg(colors::BACKGROUND),
            border_style: Style::default(),
            active_border_style: Style::new().fg(colors::ACCENT_ORANGE),
            journal_bg: Style::default().bg(colors::LBLACK),
            fail2ban_bg: Style::default().bg(Color::Rgb(48, 48, 48)),
            highlight_item_style: Style::default()
                                    .fg(Color::Black)
                                    .bg(colors::ACCENT_BLUE)
                                    .add_modifier(Modifier::BOLD),
            colors: ThemeColors { 
                default_background: colors::BACKGROUND, 
                default_map_color: Color::Rgb(139,139,141), 
                accent_blue: colors::ACCENT_BLUE, 
                accent_orange: colors::ACCENT_ORANGE,
                accent_lorange: colors::ACCENT_LORANGE,
                accent_dblue: colors::ACCENT_DBLUE,
                accent_wred: colors::ACCENT_WRED,
                accent_lpink: colors::ACCENT_LPINK,
                accent_lime: colors::ACCENT_LIME,
                lblack: colors::LBLACK,
            }
         }      
    }
}