use super::{edit, fence_of, native, reload, reload_all, screen_cursor, select};
use crate::{
    shell::{self, clip},
    store,
};
use windows::Win32::{Foundation::HWND, UI::Input::KeyboardAndMouse::*};

fn down(k: VIRTUAL_KEY) -> bool {
    (unsafe { GetKeyState(k.0 as i32) }) < 0
}

fn paste(h: HWND) {
    let (Some((files, cut)), Some(f)) = (clip::get(), fence_of(h)) else { return };
    if shell::transfer(h, &files, &store::tab_dir(f.active()), cut) {
        reload_all();
    }
}

pub(super) fn handle(h: HWND, vk: u16) -> bool {
    let (ctrl, shift, k) = (down(VK_CONTROL), down(VK_SHIFT), VIRTUAL_KEY(vk));
    let sel = select::selection(h);
    match k {
        VK_DELETE if !sel.is_empty() => {
            if shell::delete(h, &sel, shift) {
                reload(h);
            }
        }
        VK_F2 => sel.first().iter().for_each(|p| edit::item(h, p)),
        VK_RETURN => sel.iter().for_each(|p| shell::open(p)),
        VK_F5 => reload(h),
        VK_ESCAPE => select::replace(h, Default::default()),
        VK_A if ctrl => select::all(h),
        VK_C if ctrl && !sel.is_empty() => {
            clip::set(&sel, false);
        }
        VK_X if ctrl && !sel.is_empty() => {
            clip::set(&sel, true);
        }
        VK_V if ctrl => paste(h),
        VK_APPS | VK_F10 if (k == VK_APPS || shift) && !sel.is_empty() => {
            native::show(h, &sel, screen_cursor());
        }
        _ => return false,
    }
    true
}
