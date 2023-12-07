

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum Mode {
  #[default]
  Normal,
  TakeAction,
  Processing,
  Query,
  ConfirmClear,
  SetIOCapacity,
  Ban,
  Unban,
}


#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DrawMode {
  #[default]
  Sticky,
  Decaying,
  All,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum DisplayMode {
  #[default]
  Normal,
  Help,
  Query,
  Stats,
  ConfirmClear,
  SetIOCapacity,
  Ban,
  Unban,
}

#[derive(Default, Copy, Clone, PartialEq, Eq)]
pub enum IOMode {
  #[default]
  Follow, // will jump to freshly received IP
  Static, // will stay at selected IP
}
