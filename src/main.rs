pub mod app;
pub mod color;
pub mod draw;
pub mod utils;
pub mod widget;

pub mod clock;
pub mod workspaces;

use clap::Parser;

/// Simple program to greet a person
#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// The timestamp of the last update
    #[arg(short, long)]
    updated_last: Option<i64>,
}

pub fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("BAR_WLRS_LOG")
        .init();

    let args = Args::parse();
    log::info!("args: {args:?}");
    //use clap::{arg, command};
    //let matches = command!() // requires `cargo` feature
    //    .arg(arg!(-u--"last-updated" <TIME_STAMP>).required(false))
    //    .get_matches();

    let (mut app, mut event_queue) = app::App::new(args);

    app.run_queue(&mut event_queue);
}
