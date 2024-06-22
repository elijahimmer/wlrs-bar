pub mod draw;
pub mod utils;
pub mod widget;

pub mod app;

#[cfg(feature = "battery")]
pub mod battery;
#[cfg(feature = "clock")]
pub mod clock;
#[cfg(feature = "updated-last")]
pub mod updated_last;
#[cfg(feature = "workspaces")]
pub mod workspaces;

use clap::Parser;
use std::path::PathBuf;

/// A Hyprland Status Bar for me :)
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
pub struct Args {
    #[arg(long, value_name = "PATH")]
    font_path: Option<PathBuf>,

    #[arg(long, default_value_t = 0, value_name = "INDEX")]
    font_index: u32,

    /// The timestamp of the last update
    #[cfg(feature = "updated-last")]
    #[arg(short, long, value_name = "TIME_STAMP")]
    updated_last: Option<i64>,

    /// the path to the battery's device folder
    #[cfg(feature = "battery")]
    #[arg(short, long, value_name = "PATH")]
    battery_path: Option<PathBuf>,

    /// how height the bar should be
    #[arg(long, default_value_t = 28)]
    height: u32,

    /// how wide the bar should be (0 for screen width)
    #[arg(long, default_value_t = 0)]
    width: u32,
}

pub fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("BAR_WLRS_LOG")
        .init();

    let args = Args::parse();

    let (mut app, mut event_queue) = app::App::new(args);

    app.run_queue(&mut event_queue);
}
