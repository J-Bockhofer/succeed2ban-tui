//! Enums from Stats.
//! Represent Modes / States

/// Input Modes
#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
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
pub enum SortMode {
  #[default]
  Alphabetical,
  NumWarns,
  Blocked,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum SortState {
  #[default]
  Alphabetical,
  NumWarns,
  Blocked,
  AlphabeticalRev,
  NumWarnsRev,
  BlockedRev,
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
