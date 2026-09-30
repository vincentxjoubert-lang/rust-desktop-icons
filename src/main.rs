#![windows_subsystem = "windows"]

mod app;
mod domain;
mod fence;
mod i18n;
mod render;
mod shell;
mod store;
mod update;
mod win;

fn main() {
    app::run();
}
