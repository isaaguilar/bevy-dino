mod app;
mod assets;
mod camera;
#[cfg(feature = "dev")]
mod dev_tools;
mod game;

fn main() {
    app::start();
}
