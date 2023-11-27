
use ratatui::{style::Style, prelude::Color};
use regex::Regex;

use std::iter::Map;
//use std::collections::HashMap;

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



pub struct Theme {
    // Text Styling
    pub word_style_map: WordStyleMap,
    pub regex_style_map: RegexStyleMap,
    pub default_text_style: Style,
    // UI Styling
    pub border_style: Style,
    pub active_border_style: Style,
} 

impl Theme {
    pub fn new(word_style_map: WordStyleMap, 
        regex_style_map: RegexStyleMap, 
        default_text_style: Style, 
        border_style: Style,
        active_border_style: Style,) -> Self 
        {
        Theme { word_style_map, regex_style_map, default_text_style, border_style, active_border_style}
    }
}

impl Default for Theme {
    fn default() -> Self {
        //Theme::new()
        Theme { word_style_map: WordStyleMap{ word_styles: vec![
                    WordStylePair::new(String::from("Found"), Style::default().fg(Color::LightCyan)),
                    WordStylePair::new(String::from("Ban"), Style::default().fg(Color::LightYellow)),
                    WordStylePair::new(String::from("INFO"), Style::default().fg(Color::LightCyan)),
                    WordStylePair::new(String::from("WARNING"), Style::default().fg(Color::Yellow)),
                    WordStylePair::new(String::from("NOTICE"), Style::default().fg(Color::LightGreen)),
                    //WordStylePair::new(String::from("[sshd]"), Style::default().fg(Color::Rgb(138, 43, 226))),
                ]
            }, 
            regex_style_map: RegexStyleMap{ regex_styles: vec![
                RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(Color::LightRed)),
            ]},
            default_text_style: Style::default().fg(Color::White),
            border_style: Style::default(),
            active_border_style: Style::new().fg(Color::LightBlue),
         }      
    }
}