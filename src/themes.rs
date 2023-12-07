
use color_eyre::owo_colors::OwoColorize;
use ratatui::{style::Style, prelude::Color, prelude::Modifier};
use regex::Regex;

use::serde::{Deserialize, Serialize};

use std::iter::Map;
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

#[derive(Default, Clone)]
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
    pub ddblue: Color,

}

impl ThemeColors {
    pub fn new() -> Self {
        ThemeColors::default()
    }
}
#[derive(Clone)]
pub struct AppColors {
    // Background
    pub background_darkest: Color,
    pub background_mid: Color,
    pub background_brightest: Color,

    // 
    pub text_color: Color,
    pub border_color: Color,

    pub map_color: Color,
    pub warn_color: Color,
    pub confirm_color: Color,

    pub accent_color_a: Color,
    pub accent_color_a_var: Color,

    pub accent_color_b_dark: Color,
    pub accent_color_b_mid: Color,
    pub accent_color_b_bright: Color,

    pub background_text_bright: Color,
    pub background_text_dark: Color,

}

impl Default for AppColors {
  fn default() -> Self {
      AppColors { 
        background_darkest: colors::LBLACK, 
        background_mid: colors::BACKGROUND,  
        background_brightest: Color::Rgb(48, 48, 48), 
        text_color: Color::White, 
        border_color: Color::White, 
        map_color: Color::Rgb(139,139,141), 
        warn_color: colors::ACCENT_WRED, 
        confirm_color: colors::ACCENT_LIME, 
        accent_color_a: colors::ACCENT_ORANGE, 
        accent_color_a_var: colors::ACCENT_LORANGE, 
        accent_color_b_dark: colors::DDBLUE, 
        accent_color_b_mid: colors::ACCENT_DBLUE, 
        accent_color_b_bright: colors::ACCENT_BLUE, 
        background_text_bright: Color::Rgb(48, 48, 48),
        background_text_dark: colors::EMPTY,
       }
  }
}

#[derive(Clone)]
pub struct AppStyles {
  border_style: Style,
  active_border_style: Style,
  highlight_item_style: Style,
}
impl Default for AppStyles {
  fn default() -> Self {
    let default_colors = AppColors::default(); 

    AppStyles { 
      border_style: Style::new()
        .bg(default_colors.background_darkest)
        .fg(default_colors.border_color), 
      active_border_style: Style::new()
        .bg(default_colors.background_darkest)
        .fg(default_colors.accent_color_a),
      highlight_item_style: Style::new()
        .fg(default_colors.background_text_dark)
        .bg(default_colors.accent_color_b_bright)
        .add_modifier(Modifier::BOLD) }
  }
}


#[derive(Clone)]
pub struct Theme {
    // Text Styling
    pub word_style_map: WordStyleMap,
    pub regex_style_map: RegexStyleMap,


    // Colors
    pub colors_app: AppColors,
    pub styles_app: AppStyles,

    pub default_text_style: Style,
    pub username_style: Style,
    pub journal_bg: Style,
    pub fail2ban_bg: Style,
    pub selected_ip_bg: Style,
    // Multi purpose Regex
    pub ipregex: Regex,

    // UI Styling
    pub default_background: Style,
    pub border_style: Style,
    pub active_border_style: Style,
    pub highlight_item_style: Style,
    // Const Colors
    pub colors: ThemeColors,

    

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

/* impl Theme {
    pub fn new(word_style_map: WordStyleMap, 
        regex_style_map: RegexStyleMap, 
        default_text_style: Style, 
        username_style:Style,
        default_background: Style,
        border_style: Style,
        active_border_style: Style,
        journal_bg: Style,
        fail2ban_bg: Style,
        selected_ip_bg: Style,
        ipregex: Regex,
        highlight_item_style: Style, 
        colors: ThemeColors,
        decay_time: tokio::time::Duration,
        symbol_db: String,
        symbol_reqwest: String,
        symbol_block: String,
        symbol_error: String,
        symbol_unblock: String,
        symbol_ban: String,
        ) -> Self 

        {
        Theme { word_style_map, regex_style_map, default_text_style, username_style, default_background, border_style, active_border_style, 
            journal_bg,
            fail2ban_bg,
            selected_ip_bg,
            ipregex,
            highlight_item_style,
            colors,
            decay_time,
            symbol_db,
            symbol_reqwest,
            symbol_block,
            symbol_error,
            symbol_unblock,
            symbol_ban,

        }
    }

/*     pub fn default_light() -> Self {
        Theme { word_style_map: WordStyleMap{ word_styles: vec![
            WordStylePair::new(String::from("Found"), Style::default().fg(colors::ACCENT_ORANGE)),
            WordStylePair::new(String::from("Ban"), Style::default().fg(colors::ACCENT_ORANGE)),
            WordStylePair::new(String::from("INFO"), Style::default().fg(colors::ACCENT_DBLUE)),
            WordStylePair::new(String::from("WARNING"), Style::default().fg(colors::ACCENT_LPINK)),
            WordStylePair::new(String::from("NOTICE"), Style::default().fg(colors::ACCENT_LIME)),
            WordStylePair::new(String::from("user"), Style::default().fg(colors::ACCENT_BLUE)),
            WordStylePair::new(String::from("fatal:"), Style::default().fg(colors::ACCENT_ORANGE)),
            WordStylePair::new(String::from("Accepted"), Style::default().fg(colors::ACCENT_LIME)),
        ]
    }, 
    regex_style_map: RegexStyleMap{ regex_styles: vec![
        RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // IP v4
        RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // Timestamp HH:MM:SS
        //RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // Timestamp HH:MM:SS
    ]},
    default_text_style: Style::default().fg(colors::LBLACK),
    username_style: Style::default().fg(colors::ACCENT_ORANGE),
    default_background: Style::default().bg(colors::LIGHT_BACKGROUND_2),
    border_style: Style::default(),
    active_border_style: Style::new().fg(colors::BBLUE),
    journal_bg: Style::default().bg(colors::LBLACK),
    fail2ban_bg: Style::default().bg(Color::Rgb(48, 48, 48)),
    selected_ip_bg: Style::default().bg(colors::EMPTY),
    ipregex: Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(),
    highlight_item_style: Style::default()
                            .fg(Color::Black)
                            .bg(colors::ACCENT_BLUE)
                            .add_modifier(Modifier::BOLD),
    colors: ThemeColors { 
        default_background: colors::LIGHT_BACKGROUND, 
        default_map_color: colors::LBLACK, 
        accent_blue: colors::DPURP, 
        accent_orange: colors::ACCENT_WRED,
        accent_lorange: colors::ACCENT_WRED,
        accent_dblue: colors::DDPURP,
        accent_wred: colors::ACCENT_WRED,
        accent_lpink: colors::ACCENT_LPINK,
        accent_lime: colors::ACCENT_LIME,
        lblack: colors::LIGHT_BACKGROUND_2,
        ddblue: colors::DDBLUE,
    },
    decay_time: tokio::time::Duration::from_secs(10),
 }        
    } */


} */

impl Default for Theme {
    fn default() -> Self {
        //Theme::new()
        let colors = AppColors::default();
        let styles = AppStyles::default();
        Theme { word_style_map: 
          WordStyleMap{ word_styles: vec![
                    WordStylePair::new(String::from("Found"), Style::default().fg(colors.accent_color_a)),
                    WordStylePair::new(String::from("Ban"), Style::default().fg(colors.accent_color_a)),
                    WordStylePair::new(String::from("INFO"), Style::default().fg(colors.accent_color_b_mid)),
                    WordStylePair::new(String::from("WARNING"), Style::default().fg(colors::ACCENT_LPINK)),
                    WordStylePair::new(String::from("NOTICE"), Style::default().fg(colors.confirm_color)),
                    WordStylePair::new(String::from("user"), Style::default().fg(colors.accent_color_b_bright)),
                    WordStylePair::new(String::from("fatal:"), Style::default().fg(colors.accent_color_a)),
                    WordStylePair::new(String::from("Accepted"), Style::default().fg(colors.confirm_color)),
                ]
            }, 
            regex_style_map: 
            RegexStyleMap{ regex_styles: vec![
                RegexStylePair::new(Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(), Style::default().fg(colors.accent_color_b_bright)), // IP v4
                RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors.accent_color_b_bright)), // Timestamp HH:MM:SS
                //RegexStylePair::new(Regex::new(r"(\d{2}:\d{2}:\d{2})").unwrap(), Style::default().fg(colors::ACCENT_BLUE)), // Timestamp HH:MM:SS
            ]},
            //default_text_style: Style::default().fg(Color::White),
            default_text_style: Style::default().fg(colors.text_color),
            username_style: Style::default().fg(colors.accent_color_a),
            default_background: Style::default().bg(colors.background_mid),
            border_style: styles.border_style,
            active_border_style: styles.active_border_style,
            journal_bg: Style::default().bg(colors.background_darkest),
            fail2ban_bg: Style::default().bg(colors.background_text_bright),
            selected_ip_bg: Style::default().bg(colors.background_text_dark),
            ipregex: Regex::new(r"(\d{1,3}\.\d{1,3}\.\d{1,3}\.\d{1,3})").unwrap(),
            highlight_item_style: styles.highlight_item_style,
            colors: ThemeColors { 
                default_background: colors.background_mid, 
                default_map_color: colors.map_color, 
                accent_blue: colors.accent_color_b_bright, 
                accent_orange: colors.accent_color_a,
                accent_lorange: colors.accent_color_a_var,
                accent_dblue: colors.accent_color_b_mid,
                accent_wred: colors.warn_color,
                accent_lpink: colors::ACCENT_LPINK,
                accent_lime: colors.confirm_color,
                lblack: colors.background_darkest,
                ddblue: colors.accent_color_b_dark,
            },
            decay_time: tokio::time::Duration::from_secs(10),
            symbol_db: String::from("¤"), // 💽 💾 ⏫ ⌂
            symbol_reqwest: String::from("┬"), // 🗺  🌍 ⏬ ❓ ❎ 
            symbol_block: String::from("✋"), // 👊 // 👏  👌 👂 ⛩ 👓 📈  🔛 🔃
            symbol_error: String::from("⚠️"), // ⚠️ 👨‍🔧 🤖  
            symbol_unblock: String::from("🔛"),
            symbol_ban: String::from("❌"),

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
            ThemeContainer::new("DefaultDark".to_string(), Theme::default()),
            //ThemeContainer::new("DefaultLight".to_string(), Theme::default_light()),

        ] }

    }
}