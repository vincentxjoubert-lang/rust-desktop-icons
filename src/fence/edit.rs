use super::{TITLE, cell, label_font, metrics, reload, title_font, update};
use crate::{
    app::with,
    domain::{grid, names},
    i18n::T,
    report, store,
    win::*,
};
use std::{fs, path::PathBuf};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::{ClientToScreen, HFONT},
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
    let Some(parent) = with(|a| a.views.iter().find(|v| v.edit.as_ref().is_some_and(|e| e.0 == msg.hwnd)).map(|v| v.hwnd)).flatten() else {
        return false;
    };
    finish(parent, k == VK_RETURN.0);
    true
}

fn open(h: HWND, r: RECT, text: &str, font: HFONT, target: Option<PathBuf>) {
    let mut p = POINT { x: r.left, y: r.top };
    unsafe {
        let _ = ClientToScreen(h, &mut p);
        let style = WS_POPUP | WS_VISIBLE | WS_BORDER | WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL) as u32);
        let (w, ht) = (r.right - r.left, r.bottom - r.top);
        let Ok(e) = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_TOPMOST,
            w!("EDIT"),
            PCWSTR(wide(text).as_ptr()),
            style,
            p.x,
            p.y,
            w,
            ht,
            Some(h),
            None,
            Some(inst()),
            None,
        ) else {
            return;
        };
        SendMessageW(e, WM_SETFONT, Some(WPARAM(font.0 as usize)), Some(LPARAM(1)));
        SendMessageW(e, EM_LIMITTEXT, Some(WPARAM(255)), None);
        SendMessageW(e, EM_SETSEL, Some(WPARAM(0)), Some(LPARAM(-1)));
        with(|a| a.view(h).map(|v| v.edit = Some((e, target))));
        let _ = SetForegroundWindow(e);
        let _ = SetFocus(Some(e));
    }
}

pub(super) fn title(h: HWND) {
    let Some((text, font)) = with(|a| Some((a.fence_of(h)?.active().title.clone(), title_font(a, h)))).flatten() else { return };
    let (r, t, pad) = (client_rect(h), scale(h, TITLE), scale(h, 5));
    open(h, RECT { left: pad, top: pad, right: r.right - pad, bottom: t - pad }, &text, font, None);
}

pub(super) fn item(h: HWND, path: &PathBuf) {
    if crate::shell::recycle::is(path) {
        return;
    }
    let s = |v| scale(h, v);
    let (cell, icon) = metrics(h);
    let w = client_rect(h).right;
    let Some((text, font, origin)) = with(|a| {
        let font = label_font(a, h);
        let v = a.view(h)?;
        let i = v.items.iter().position(|it| &it.path == path)?;
        let (x, y) = grid::origin(i, w, cell);
        Some((String::from_utf16_lossy(&v.items[i].name), font, (x, y + s(TITLE) + s(4) - v.scroll)))
    })
    .flatten() else {
        return;
    };
    let r = cell::label_at(origin, cell, icon, s);
    open(
        h,
        RECT { top: r.top - s(2), bottom: r.top + s(22), left: r.left - s(10), right: r.right + s(10) },
        &text,
        font,
        Some(path.clone()),
    );
}

pub(super) fn finish(h: HWND, commit: bool) {
    let Some((e, target)) = with(|a| a.view(h)?.edit.take()).flatten() else {
        return;
    };
    let mut buf = [0u16; 260];
    let n = unsafe { GetWindowTextW(e, &mut buf) } as usize;
    unsafe { DestroyWindow(e).ok() };
    let text = String::from_utf16_lossy(&buf[..n]);
    if !commit {
        return;
    }
    match target {
        None if !text.trim().is_empty() => {
            update(h, |f| f.active_mut().title = text.trim().chars().filter(|c| !c.is_control()).take(64).collect())
        }
        Some(path) => {
            let shown = with(|a| a.view(h)?.items.iter().find(|i| i.path == path).map(|i| String::from_utf16_lossy(&i.name))).flatten();
            let file = store::name(&path);
            if let Some(name) = names::renamed(&file, shown.as_deref().unwrap_or(&file), &text) {
                let dest = path.with_file_name(&name);
                let result =
                    if dest.exists() { Err(std::io::Error::from(std::io::ErrorKind::AlreadyExists)) } else { fs::rename(&path, &dest) };
                match result {
                    Ok(()) => reload(h),
                    Err(err) => report::alert(T::ErrRename, &format!("{file} → {name} ({err})")),
                }
            }
        }
        None => {}
    }
}
