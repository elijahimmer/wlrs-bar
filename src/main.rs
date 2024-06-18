pub mod app;
pub mod color;
pub mod draw;
pub mod utils;
pub mod widget;

pub mod clock;
pub mod workspaces;

pub fn main() {
    pretty_env_logger::formatted_builder()
        .filter_level(log::LevelFilter::Info)
        .parse_env("BAR_WLRS_LOG")
        .init();

    let (mut app, mut event_queue) = app::App::new();

    app.run_queue(&mut event_queue);
}
