pub mod app;
pub mod color;
pub mod draw;
pub mod utils;
pub mod widget;

pub mod clock;
pub mod workspaces;

pub fn main() {
    env_logger::init();

    //use workspaces::utils::*;

    //println!("{:?}", get_workspaces().unwrap());

    let (mut app, mut event_queue) = app::App::new();

    app.run_queue(&mut event_queue);
}
