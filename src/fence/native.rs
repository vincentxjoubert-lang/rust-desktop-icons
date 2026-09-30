use super::edit;
use crate::{app::with, shell::menu};
use std::path::PathBuf;
use windows::Win32::{
    Foundation::{HWND, LPARAM, LRESULT, WPARAM},
    UI::Input::KeyboardAndMouse::{GetKeyState, VK_SHIFT},
};

pub(super) fn show(h: HWND, paths: &[PathBuf], at: (i32, i32)) {
    let extended = (unsafe { GetKeyState(VK_SHIFT.0 as i32) }) < 0;
    let Some(native) = menu::build(paths, extended) else { return };
    with(|a| a.view(h).map(|v| v.native = Some(native.cm.clone())));
    let id = native.track(h, at);
    with(|a| a.view(h).map(|v| v.native = None));
    let Some(id) = id else { return };
    if native.verb(id).eq_ignore_ascii_case("rename") {
        if let Some(p) = paths.first() {
            edit::item(h, p);
        }
    } else {
        native.invoke(id, h, at);
    }
}

pub(super) fn forward(h: HWND, msg: u32, wp: WPARAM, lp: LPARAM) -> Option<LRESULT> {
    let cm = with(|a| a.view(h)?.native.clone()).flatten()?;
    menu::forward(&cm, msg, wp, lp)
}
