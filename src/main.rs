mod app;
mod assets;
mod camera;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;
mod menu;
mod util;

fn main() {
    app::start();
}
