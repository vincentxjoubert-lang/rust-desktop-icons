#![windows_subsystem = "windows"]

mod app;
mod domain;
mod fence;
mod i18n;
mod layered;
mod prefs;
mod render;
mod report;
mod rules;
mod settings;
mod shell;
mod store;
mod tray;
mod uninstall;
mod update;
mod whatsnew;
mod win;

fn main() {
    app::run();
}
