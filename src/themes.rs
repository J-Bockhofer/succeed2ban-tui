
use color_eyre::owo_colors::OwoColorize;
use ratatui::{style::Style, prelude::Color, prelude::Modifier};
use regex::Regex;

use::serde::{Deserialize, Serialize};

use std::iter::Map;

use self::colors::ColorRGB;
//use std::collections::HashMap;

pub mod colors;

#[derive(Default, Clone)]
pub struct WordStylePair {
    pub word: String,
    pub style: Style,
}

impl WordStylePair {
    pub fn new(word: String, style: Style) -> Self {
        WordStylePair { word, style }
    }
}


#[derive(Clone)]
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
#[derive(Default, Clone)]
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

#[derive(Default, Clone)]
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


#[derive(Clone)]
pub struct AppColors {
    // Background
    pub background_darkest: ColorRGB,
    pub background_mid: ColorRGB,
    pub background_brightest: ColorRGB,

    // 
    pub text_color: ColorRGB,
    pub border_color: ColorRGB,

    pub map_color: ColorRGB,
    pub warn_color: ColorRGB,
    pub error_color: ColorRGB,
    pub confirm_color: ColorRGB,

    pub accent_color_a: ColorRGB,
    pub accent_color_a_var: ColorRGB,

    pub accent_color_b_dark: ColorRGB,
    pub accent_color_b_mid: ColorRGB,
    pub accent_color_b_bright: ColorRGB,

    pub background_text_bright: ColorRGB,
    pub background_text_dark: ColorRGB,

}

impl Default for AppColors {
  fn default() -> Self {
      AppColors { 
        background_darkest: ColorRGB::from_color(colors::LBLACK).unwrap(), 
        background_mid: ColorRGB::from_color(colors::BACKGROUND).unwrap(),  
        background_brightest: ColorRGB::from_color(Color::Rgb(48, 48, 48)).unwrap(), 
        text_color: ColorRGB::from_color(Color::White).unwrap(), 
        border_color: ColorRGB::from_color(Color::White).unwrap(), 
        map_color: ColorRGB::from_color(Color::Rgb(139,139,141)).unwrap(), 
        warn_color: ColorRGB::from_color(colors::ACCENT_WRED).unwrap(), 
        error_color: ColorRGB::from_color(colors::ACCENT_LPINK).unwrap(),
        confirm_color: ColorRGB::from_color(colors::ACCENT_LIME).unwrap(), 
        accent_color_a: ColorRGB::from_color(colors::ACCENT_ORANGE).unwrap(), 
        accent_color_a_var: ColorRGB::from_color(colors::ACCENT_LORANGE).unwrap(), 
        accent_color_b_dark: ColorRGB::from_color(colors::DDBLUE).unwrap(), 
        accent_color_b_mid: ColorRGB::from_color(colors::ACCENT_DBLUE).unwrap(), 
        accent_color_b_bright: ColorRGB::from_color(colors::ACCENT_BLUE).unwrap(), 
        background_text_bright: ColorRGB::from_color(Color::Rgb(48, 48, 48)).unwrap(),
        background_text_dark: ColorRGB::from_color(colors::EMPTY).unwrap(),
       }
  }
}

impl AppColors {
    pub fn paper() -> Self{
        AppColors { 
            background_darkest: ColorRGB::from_color(ColorRGB::from_hex("#EEEECC").unwrap().shade(-0.16)).unwrap(), 
            background_mid: ColorRGB::from_color(ColorRGB::from_hex("#EEEECC").unwrap().shade(-0.16)).unwrap(),  
            background_brightest: ColorRGB::from_hex("#EEEECC").unwrap(), 
            text_color: ColorRGB::new(24,24,24), 
            border_color: ColorRGB::new(12,12,12),
            map_color: ColorRGB::from_hex("#474838").unwrap(), 
            warn_color: ColorRGB::from_hex("#d9a13a").unwrap(), 
            error_color: ColorRGB::from_hex("#a5d9d9").unwrap(),
            confirm_color: ColorRGB::from_hex("#00917b").unwrap(), 
            accent_color_a: ColorRGB::from_hex("#ac2275").unwrap(), 
            accent_color_a_var: ColorRGB::from_hex("#dc4f8b").unwrap(), 
            accent_color_b_dark: ColorRGB::from_hex("#D56A60").unwrap(), //#00a988
            accent_color_b_mid: ColorRGB::from_hex("#4CBFF1").unwrap(), // #4a90e5 4CBFF1
            accent_color_b_bright: ColorRGB::from_hex("#00a988").unwrap(),  // #00a4eb -- aggro blue
            background_text_bright: ColorRGB::from_hex("#EAEAEA").unwrap(),
            background_text_dark: ColorRGB::new(12,12,12),
        }
    }
}


#[derive(Clone)]
pub struct AppStyles {
  pub border_style: Style,
  pub active_border_style: Style,
  pub highlight_item_style: Style,
  pub default_style: Style,
}
impl Default for AppStyles {
  fn default() -> Self {
    let default_colors = AppColors::default(); 

    AppStyles { 
      border_style: Style::new()
        .bg(default_colors.background_darkest.color)
        .fg(default_colors.border_color.color), 
      active_border_style: Style::new()
        .bg(default_colors.background_darkest.color)
        .fg(default_colors.accent_color_a.color),
      highlight_item_style: Style::new()
        .fg(default_colors.background_text_dark.color)
        .bg(default_colors.accent_color_b_bright.color)
        .add_modifier(Modifier::BOLD),
      default_style: Style::new()
        .fg(default_colors.text_color.color)
        .bg(default_colors.background_mid.color),}
  }
}

impl AppStyles {

    pub fn paper() -> Self {
        let default_colors = AppColors::paper(); 

        AppStyles { 
          border_style: Style::new()
            .bg(default_colors.background_darkest.color)
            .fg(default_colors.border_color.color), 
          active_border_style: Style::new()
            .bg(default_colors.background_darkest.color)
            .fg(default_colors.accent_color_a.color),
          highlight_item_style: Style::new()
            .fg(default_colors.background_text_dark.color)
            .bg(default_colors.accent_color_b_mid.color)
            .add_modifier(Modifier::BOLD),
          default_style: Style::new()
            .fg(default_colors.text_color.color)
            .bg(default_colors.background_mid.color),}        
    }

}


#[derive(Clone)]
pub struct Theme {
    pub is_light: bool,
    // Text Styling
    pub word_style_map: WordStyleMap,
    pub regex_style_map: RegexStyleMap,

    // Colors
    pub colors_app: AppColors,
    // Styles
    pub styles_app: AppStyles,

    pub ipregex: Regex,

    // Timing
    pub decay_time: tokio::time::Duration,

    // Symbols
    pub symbol_db: String,
    pub symbol_reqwest: String,
    pub symbol_block: String,
    pub symbol_error: String,
    pub symbol_unblock: String,
    pub symbol_ban: String,
    
} 

impl Default for Theme {
    fn default() -> Self {
        //Theme::new()
        let colors = AppColors::default();
        let styles = AppStyles::default();
        Theme { is_light: false,word_style_map: 
          WordStyleMap{ word_styles: vec![
                    WordStylePair::new(String::from("Found"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("Ban"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("INFO"), Style::default().fg(colors.accent_color_b_mid.color)),
                    WordStylePair::new(String::from("WARNING"), Style::default().fg(colors.error_color.color)),
                    WordStylePair::new(String::from("NOTICE"), Style::default().fg(colors.confirm_color.color)),
                    WordStylePair::new(String::from("user"), Style::default().fg(colors.accent_color_b_bright.color)),
                    WordStylePair::new(String::from("fatal:"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("Accepted"), Style::default().fg(colors.confirm_color.color)),
                ]
            }, 
            regex_style_map: 
            RegexStyleMap{ regex_styles: vec![
                RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(colors.accent_color_b_bright.color)), // IP v4
                RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors.accent_color_b_bright.color)), // Timestamp HH:MM:SS
            ]},
            ipregex: Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(),
            decay_time: tokio::time::Duration::from_secs(10),
            symbol_db: String::from("¤"), // 💽 💾 ⏫ ⌂
            symbol_reqwest: String::from("┬"), // 🗺  🌍 ⏬ ❓ ❎ 
            symbol_block: String::from("✋"), // 👊 // 👏  👌 👂 ⛩ 👓 📈  🔛 🔃
            symbol_error: String::from("⚠️"), // ⚠️ 👨‍🔧 🤖  
            symbol_unblock: String::from("🔛"),
            symbol_ban: String::from("✋"),

            colors_app: colors,
            styles_app: styles,

         }      
    }
}

impl Theme {
    pub fn paper() -> Self {
        let colors = AppColors::paper();
        let styles = AppStyles::paper();
        Theme {is_light:true, word_style_map: 
          WordStyleMap{ word_styles: vec![
                    WordStylePair::new(String::from("Found"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("Ban"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("INFO"), Style::default().fg(colors.confirm_color.color)),
                    WordStylePair::new(String::from("WARNING"), Style::default().fg(colors.warn_color.color)),
                    WordStylePair::new(String::from("NOTICE"), Style::default().fg(colors.text_color.color)),
                    WordStylePair::new(String::from("user"), Style::default().fg(colors.accent_color_b_bright.color)),
                    WordStylePair::new(String::from("fatal:"), Style::default().fg(colors.accent_color_a.color)),
                    WordStylePair::new(String::from("Accepted"), Style::default().fg(colors.confirm_color.color)),
                ]
            }, 
            regex_style_map: 
            RegexStyleMap{ regex_styles: vec![
                RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(colors.accent_color_a.color)), // IP v4
                RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors.accent_color_a.color)), // Timestamp HH:MM:SS
            ]},
            ipregex: Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(),
            decay_time: tokio::time::Duration::from_secs(10),
            symbol_db: String::from("¤"), // 💽 💾 ⏫ ⌂
            symbol_reqwest: String::from("┬"), // 🗺  🌍 ⏬ ❓ ❎ 
            symbol_block: String::from("✋"), // 👊 // 👏  👌 👂 ⛩ 👓 📈  🔛 🔃
            symbol_error: String::from("⚠️"), // ⚠️ 👨‍🔧 🤖  
            symbol_unblock: String::from("🔛"),
            symbol_ban: String::from("✋"),

            colors_app: colors,
            styles_app: styles,

         }              


    }

}


#[derive(Default, Clone)]
pub struct ThemeContainer {
    pub name: String,
    pub theme: Theme,
}
impl ThemeContainer {
    pub fn new(name:String, theme: Theme) -> Self {
        ThemeContainer { name, theme }
    }
} 



pub struct Themes {
    pub theme_collection: Vec<ThemeContainer>,
}
impl Default for Themes {
    fn default() -> Self {
        Themes { theme_collection: vec![
            ThemeContainer::new("Dark".to_string(), Theme::default()),
            ThemeContainer::new("Paper".to_string(), Theme::paper()),
        ] }

    }
}