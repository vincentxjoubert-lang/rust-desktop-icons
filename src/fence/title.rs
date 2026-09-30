use super::{TITLE, title_font, update};
use crate::{app::with, win::*};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::ClientToScreen,
        UI::{
            Controls::{EM_LIMITTEXT, EM_SETSEL},
            Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_RETURN},
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

pub fn key(msg: &MSG) -> bool {
    let k = msg.wParam.0 as u16;
    if msg.message != WM_KEYDOWN || (k != VK_RETURN.0 && k != VK_ESCAPE.0) {
        return false;
    }
    let Some(parent) = with(|a| a.views.iter().find(|v| v.edit == Some(msg.hwnd)).map(|v| v.hwnd)).flatten() else {
        return false;
    };
    finish(parent, k == VK_RETURN.0);
    true
}

pub(super) fn rename(h: HWND) {
    let Some((title, font)) = with(|a| {
        let title = a.fence_of(h)?.active().title.clone();
        Some((title, title_font(a, h)))
    })
    .flatten() else {
        return;
    };
    let (r, t, pad) = (client_rect(h), scale(h, TITLE), scale(h, 5));
    let mut p = POINT { x: pad, y: pad };
    unsafe {
        let _ = ClientToScreen(h, &mut p);
        let style = WS_POPUP | WS_VISIBLE | WS_BORDER | WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL) as u32);
        let Ok(e) = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("EDIT"),
            PCWSTR(wide(&title).as_ptr()),
            style,
            p.x,
            p.y,
            r.right - 2 * pad,
            t - 2 * pad,
            Some(h),
            None,
            Some(inst()),
            None,
        ) else {
            return;
        };
        SendMessageW(e, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
        SendMessageW(e, EM_LIMITTEXT, Some(WPARAM(64)), None);
        SendMessageW(e, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
        with(|a| a.view(h).map(|v| v.edit = Some(e)));
        let _ = SetForegroundWindow(e);
        let _ = SetFocus(Some(e));
    }
}

pub(super) fn finish(h: HWND, commit: bool) {
    let Some(e) = with(|a| a.view(h)?.edit.take()).flatten() else {
        return;
    };
    let mut buf = [0u16; 80];
    let n = unsafe { GetWindowTextW(e, &mut buf) } as usize;
    unsafe { DestroyWindow(e).ok() };
    let text = String::from_utf16_lossy(&buf[..n]).trim().to_string();
    if commit && !text.is_empty() {
        update(h, |f| f.active_mut().title = text.chars().filter(|c| !c.is_control()).take(64).collect());
    }
}
