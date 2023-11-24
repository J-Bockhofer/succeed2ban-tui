fn main()
{
    
}

/* use std::path::PathBuf;

use anyhow::{Context, Result};
use colored::Colorize;
use directories::ProjectDirs;
use env_logger;
use tracing_subscriber::{
  self, filter::EnvFilter, prelude::__tracing_subscriber_SubscriberExt, util::SubscriberInitExt, Layer,
};

pub fn initialize_logging() -> Result<()> {
  let directory = if let Ok(s) = std::env::var("MY_APP_NAME_HERE_DATA") {
    PathBuf::from(s)
  } else if let Some(proj_dirs) = ProjectDirs::from("com", "kdheepak", "my-app-name-here") {
    proj_dirs.data_local_dir().to_path_buf()
  } else {
    let s = "Error".red().bold();
    eprintln!("{s}: Unable to find data directory for my-app-name-here");
    std::process::exit(1)
  };
  std::fs::create_dir_all(directory.clone()).context(format!("{directory:?} could not be created"))?;
  let log_path = directory.join("my-app-name-here-debug.log");
  let log_file = std::fs::File::create(&log_path)?;
  let file_subscriber = tracing_subscriber::fmt::layer()
    .with_file(true)
    .with_line_number(true)
    .with_writer(log_file)
    .with_target(false)
    .with_ansi(false)
    .with_filter(EnvFilter::from_default_env());
  tracing_subscriber::registry().with(file_subscriber).with(tui_logger::tracing_subscriber_layer()).init();
  tui_logger::set_default_level(log::LevelFilter::Info);
  Ok(())
} */