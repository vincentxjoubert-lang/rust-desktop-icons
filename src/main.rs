#![windows_subsystem = "windows"]

mod app;
mod domain;
mod fence;
mod i18n;
mod layered;
mod prefs;
mod render;
mod rules;
mod settings;
mod shell;
mod store;
mod tray;
mod update;
mod win;

fn main() {
    app::run();
}
