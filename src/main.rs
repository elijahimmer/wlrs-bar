pub mod app;
pub mod color;
pub mod draw;
pub mod utils;
pub mod widget;

pub mod clock;
pub mod workspaces;

pub fn main() {
    env_logger::Builder::from_default_env()
        //.format_module_path(true)
        .init();

    let (mut app, mut event_queue) = app::App::new();

    app.run_queue(&mut event_queue);
}
