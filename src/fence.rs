use crate::{
    app::{Item, with},
    domain::{Fence, Zone, color, grid, zone},
    i18n::T,
    shell, store,
    win::*,
};
use std::{fs, path::PathBuf};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{
            Controls::{Dialogs::*, EM_LIMITTEXT, EM_SETSEL},
            Input::KeyboardAndMouse::{SetFocus, VK_ESCAPE, VK_RETURN},
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

pub const CLASS: PCWSTR = w!("RustDesktopIcons.Fence");
const TITLE: i32 = 28;
const CELL: i32 = 84;
const ICON: i32 = 32;
const BORDER: i32 = 6;
const EN_KILLFOCUS: u32 = 0x0200;

fn xy(lp: LPARAM) -> (i32, i32) {
    (lp.0 as i16 as i32, (lp.0 >> 16) as i16 as i32)
}

fn client(h: HWND) -> RECT {
    let mut r = RECT::default();
    unsafe { GetClientRect(h, &mut r).ok() };
    r
}

fn id(h: HWND) -> Option<u64> {
    with(|a| a.view(h).map(|v| v.id)).flatten()
}

fn fence_of(h: HWND) -> Option<Fence> {
    with(|a| {
        let id = a.view(h)?.id;
        a.fence(id).cloned()
    })
    .flatten()
}

fn update(h: HWND, f: impl FnOnce(&mut Fence)) {
    with(|a| {
        let id = a.view(h)?.id;
        f(a.fence(id)?);
        a.save();
        Some(())
    });
    apply(h);
}

pub fn create(f: &Fence) -> Option<HWND> {
    unsafe {
        let owner = FindWindowW(w!("Progman"), None).ok();
        let h = CreateWindowExW(
            WS_EX_TOOLWINDOW | WS_EX_LAYERED | WS_EX_ACCEPTFILES,
            CLASS,
            PCWSTR::null(),
            WS_POPUP | WS_CLIPCHILDREN,
            f.x,
            f.y,
            f.w,
            f.h,
            owner,
            None,
            Some(inst()),
            None,
        )
        .ok()?;
        let _ = SetLayeredWindowAttributes(h, COLORREF(0), f.alpha, LWA_ALPHA);
        let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
        Some(h)
    }
}

fn apply(h: HWND) {
    let Some(f) = fence_of(h) else { return };
    let height = if f.rolled { scale(h, TITLE) } else { f.h };
    unsafe {
        let _ = SetLayeredWindowAttributes(h, COLORREF(0), f.alpha, LWA_ALPHA);
        let _ = SetWindowPos(h, None, 0, 0, f.w, height, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        let _ = InvalidateRect(Some(h), None, false);
    }
}

pub fn reload(h: HWND) {
    let Some(id) = id(h) else { return };
    let mut paths: Vec<PathBuf> = fs::read_dir(store::fence_dir(id))
        .into_iter()
        .flatten()
        .flatten()
        .map(|e| e.path())
        .filter(|p| !p.file_name().is_some_and(|n| n.eq_ignore_ascii_case("desktop.ini")))
        .collect();
    paths.sort_by_key(|p| p.file_name().map(|n| n.to_string_lossy().to_lowercase()));
    let items: Vec<Item> = paths
        .into_iter()
        .map(|path| {
            let (icon, name) = shell::info(&path);
            Item { path, name, icon }
        })
        .collect();
    with(|a| a.view(h).map(|v| v.items = items));
    let _ = unsafe { InvalidateRect(Some(h), None, false) };
}

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

fn rename(h: HWND) {
    let px = scale(h, 13);
    let Some((title, font)) = with(|a| {
        let id = a.view(h)?.id;
        let title = a.fence(id)?.title.clone();
        Some((title, a.font(px)))
    })
    .flatten() else {
        return;
    };
    let (r, t, pad) = (client(h), scale(h, TITLE), scale(h, 4));
    unsafe {
        let style = WS_CHILD | WS_VISIBLE | WS_BORDER | WINDOW_STYLE((ES_CENTER | ES_AUTOHSCROLL) as u32);
        let Ok(e) = CreateWindowExW(
            WINDOW_EX_STYLE(0),
            w!("EDIT"),
            PCWSTR(wide(&title).as_ptr()),
            style,
            pad,
            pad / 2,
            r.right - 2 * pad,
            t - pad,
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
        let _ = SetForegroundWindow(h);
        let _ = SetFocus(Some(e));
    }
}

fn finish(h: HWND, commit: bool) {
    let Some(e) = with(|a| a.view(h)?.edit.take()).flatten() else {
        return;
    };
    let mut buf = [0u16; 80];
    let n = unsafe { GetWindowTextW(e, &mut buf) } as usize;
    unsafe { DestroyWindow(e).ok() };
    let text = String::from_utf16_lossy(&buf[..n]).trim().to_string();
    if commit && !text.is_empty() {
        update(h, |f| f.title = text.chars().filter(|c| !c.is_control()).take(64).collect());
    }
}

fn pick_color(h: HWND, f: &Fence) {
    let mut custom = [COLORREF(0); 16];
    let mut cc = CHOOSECOLORW {
        lStructSize: size_of::<CHOOSECOLORW>() as u32,
        hwndOwner: h,
        rgbResult: COLORREF(f.color),
        lpCustColors: custom.as_mut_ptr(),
        Flags: CC_RGBINIT | CC_FULLOPEN,
        ..Default::default()
    };
    if unsafe { ChooseColorW(&mut cc) }.as_bool() {
        update(h, |f| f.color = cc.rgbResult.0 & 0xFF_FFFF);
    }
}

fn delete(h: HWND, f: &Fence) {
    let Some((q, rtl)) = with(|a| (a.t(T::ConfirmDelete), a.rtl())) else {
        return;
    };
    if msgbox(Some(h), q, MB_YESNO | MB_ICONQUESTION, rtl) != IDYES {
        return;
    }
    let dir = store::fence_dir(f.id);
    if let Some(desk) = shell::desktop() {
        for e in fs::read_dir(&dir).into_iter().flatten().flatten() {
            let _ = store::move_into(&e.path(), &desk);
        }
    }
    if fs::remove_dir(&dir).is_err() && dir.exists() {
        return reload(h);
    }
    with(|a| {
        a.views.retain(|v| v.hwnd != h);
        a.cfg.fences.retain(|x| x.id != f.id);
        a.save();
    });
    unsafe { DestroyWindow(h).ok() };
}

fn item_at(h: HWND, (x, y): (i32, i32)) -> Option<PathBuf> {
    let (w, t, cell) = (client(h).right, scale(h, TITLE), scale(h, CELL));
    with(|a| {
        let v = a.view(h)?;
        let i = grid::index_at((x, y - t + v.scroll), w, cell, v.items.len())?;
        (y >= t).then(|| v.items[i].path.clone())
    })
    .flatten()
}

fn context(h: HWND) {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
        let _ = ScreenToClient(h, &mut p);
    }
    let target = item_at(h, (p.x, p.y));
    let Some(f) = fence_of(h) else { return };
    let Some((m, rtl)) = with(|a| {
        let m = menu();
        if target.is_some() {
            item(m, 30, a.t(T::Open), false);
            item(m, 31, a.t(T::Restore), false);
            separator(m);
        }
        let o = menu();
        for k in 0..8u8 {
            item(o, 20 + k as usize, &format!("{}%", 30 + k * 10), f.alpha == opacity(k));
        }
        item(m, 10, a.t(T::Rename), false);
        item(m, 11, a.t(T::Color), false);
        submenu(m, a.t(T::Opacity), o);
        item(m, 12, a.t(T::Roll), f.rolled);
        item(m, 13, a.t(T::OpenFolder), false);
        separator(m);
        item(m, 14, a.t(T::DeleteFence), false);
        separator(m);
        item(m, 15, a.t(T::NewFence), false);
        (m, a.rtl())
    }) else {
        return;
    };
    match popup(h, m, rtl) {
        10 => rename(h),
        11 => pick_color(h, &f),
        12 => update(h, |f| f.rolled ^= true),
        13 => {
            let d = store::fence_dir(f.id);
            let _ = fs::create_dir_all(&d);
            shell::open(&d);
        }
        14 => delete(h, &f),
        15 => crate::app::new_fence(),
        k @ 20..=27 => update(h, |f| f.alpha = opacity(k as u8 - 20)),
        30 => target.iter().for_each(|p| shell::open(p)),
        31 => {
            if let (Some(p), Some(desk)) = (target, shell::desktop()) {
                let _ = store::move_into(&p, &desk);
                reload(h);
            }
        }
        _ => {}
    }
}

fn opacity(k: u8) -> u8 {
    ((30 + k as u32 * 10) * 255 / 100) as u8
}

fn drop_files(h: HWND, d: HDROP) {
    let n = unsafe { DragQueryFileW(d, u32::MAX, None) };
    let paths: Vec<PathBuf> = (0..n)
        .map(|i| {
            let mut b = vec![0u16; unsafe { DragQueryFileW(d, i, None) } as usize + 1];
            let len = unsafe { DragQueryFileW(d, i, Some(&mut b)) } as usize;
            PathBuf::from(String::from_utf16_lossy(&b[..len]))
        })
        .collect();
    unsafe { DragFinish(d) };
    let (Some(id), Some(desk)) = (id(h), shell::desktop()) else {
        return;
    };
    let (dir, fences) = (store::fence_dir(id), store::root().join("fences"));
    for p in paths.iter().filter(|p| p.parent().is_some_and(|q| q == desk || q.parent() == Some(fences.as_path())) && !p.starts_with(&dir))
    {
        let _ = store::move_into(p, &dir);
    }
    with(|a| a.views.iter().map(|v| v.hwnd).collect::<Vec<_>>()).into_iter().flatten().for_each(reload);
}

fn paint(h: HWND) {
    unsafe {
        let mut ps = PAINTSTRUCT::default();
        let dc = BeginPaint(h, &mut ps);
        let rc = client(h);
        let mdc = CreateCompatibleDC(Some(dc));
        let bmp = CreateCompatibleBitmap(dc, rc.right, rc.bottom);
        let old = SelectObject(mdc, bmp.into());
        draw(h, mdc, rc);
        let _ = BitBlt(dc, 0, 0, rc.right, rc.bottom, Some(mdc), 0, 0, SRCCOPY);
        SelectObject(mdc, old);
        let _ = DeleteObject(bmp.into());
        let _ = DeleteDC(mdc);
        let _ = EndPaint(h, &ps);
    }
}

unsafe fn fill(dc: HDC, r: &RECT, c: u32) {
    let b = CreateSolidBrush(COLORREF(c));
    FillRect(dc, r, b);
    let _ = DeleteObject(b.into());
}

fn draw(h: HWND, dc: HDC, rc: RECT) {
    let (t, cell, icon, px) = (scale(h, TITLE), scale(h, CELL), scale(h, ICON), scale(h, 13));
    with(|a| unsafe {
        let font = a.font(px);
        let id = a.view(h)?.id;
        let f = a.fence(id)?.clone();
        let v = a.view(h)?;
        fill(dc, &rc, f.color);
        fill(dc, &RECT { bottom: t, ..rc }, color::shade(f.color, 24));
        SelectObject(dc, font.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(color::contrast(f.color)));
        let mut title: Vec<u16> = f.title.encode_utf16().collect();
        DrawTextW(
            dc,
            &mut title,
            &mut RECT { left: t / 2, right: rc.right - t / 2, bottom: t, ..rc },
            DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX,
        );
        if f.rolled {
            return Some(());
        }
        IntersectClipRect(dc, 0, t, rc.right, rc.bottom);
        v.scroll = v.scroll.clamp(0, grid::max_scroll(v.items.len(), rc.right, rc.bottom - t, cell));
        for (i, it) in v.items.iter_mut().enumerate() {
            let (x, y) = grid::origin(i, rc.right, cell);
            let y = y + t - v.scroll;
            if y + cell < t || y > rc.bottom {
                continue;
            }
            let _ = DrawIconEx(dc, x + (cell - icon) / 2, y + icon / 5, it.icon, icon, icon, 0, None, DI_NORMAL);
            let mut r = RECT { left: x + 2, top: y + icon + icon / 3, right: x + cell - 2, bottom: y + cell };
            DrawTextW(dc, &mut it.name, &mut r, DT_CENTER | DT_WORDBREAK | DT_END_ELLIPSIS | DT_NOPREFIX | DT_EDITCONTROL);
        }
        Some(())
    });
}

fn persist(h: HWND) {
    let mut r = RECT::default();
    unsafe { GetWindowRect(h, &mut r).ok() };
    update(h, |f| {
        (f.x, f.y, f.w) = (r.left, r.top, r.right - r.left);
        if !f.rolled {
            f.h = r.bottom - r.top;
        }
    });
}

pub unsafe extern "system" fn proc(h: HWND, m: u32, wp: WPARAM, lp: LPARAM) -> LRESULT {
    match m {
        WM_NCHITTEST => {
            let (mut r, (x, y)) = (RECT::default(), xy(lp));
            let _ = GetWindowRect(h, &mut r);
            let rolled = fence_of(h).is_some_and(|f| f.rolled);
            let z = zone((r.right - r.left, r.bottom - r.top), (x - r.left, y - r.top), scale(h, BORDER), scale(h, TITLE), rolled);
            let ht = match z {
                Zone::Client => HTCLIENT,
                Zone::Caption => HTCAPTION,
                Zone::Left => HTLEFT,
                Zone::Right => HTRIGHT,
                Zone::Top => HTTOP,
                Zone::Bottom => HTBOTTOM,
                Zone::TopLeft => HTTOPLEFT,
                Zone::TopRight => HTTOPRIGHT,
                Zone::BottomLeft => HTBOTTOMLEFT,
                Zone::BottomRight => HTBOTTOMRIGHT,
            };
            return LRESULT(ht as isize);
        }
        WM_PAINT => paint(h),
        WM_ERASEBKGND => return LRESULT(1),
        WM_SIZE => drop(unsafe { InvalidateRect(Some(h), None, false) }),
        WM_WINDOWPOSCHANGING => {
            let p = unsafe { &mut *(lp.0 as *mut WINDOWPOS) };
            p.hwndInsertAfter = HWND_BOTTOM;
            p.flags &= !SWP_NOZORDER;
        }
        WM_GETMINMAXINFO => {
            let i = unsafe { &mut *(lp.0 as *mut MINMAXINFO) };
            i.ptMinTrackSize = POINT { x: scale(h, Fence::MIN.0), y: scale(h, TITLE) };
        }
        WM_EXITSIZEMOVE => persist(h),
        WM_DPICHANGED => {
            let r = unsafe { &*(lp.0 as *const RECT) };
            unsafe { SetWindowPos(h, None, r.left, r.top, r.right - r.left, r.bottom - r.top, SWP_NOZORDER | SWP_NOACTIVATE).ok() };
        }
        WM_NCLBUTTONDBLCLK if wp.0 as u32 == HTCAPTION => update(h, |f| f.rolled ^= true),
        WM_LBUTTONDBLCLK => item_at(h, xy(lp)).iter().for_each(|p| shell::open(p)),
        WM_CONTEXTMENU => context(h),
        WM_MOUSEWHEEL => {
            let step = (wp.0 >> 16) as i16 as i32 * scale(h, CELL) / 120;
            with(|a| a.view(h).map(|v| v.scroll -= step));
            let _ = unsafe { InvalidateRect(Some(h), None, false) };
        }
        WM_DROPFILES => drop_files(h, HDROP(wp.0 as _)),
        WM_ACTIVATE if wp.0 as u32 & 0xFFFF != WA_INACTIVE => reload(h),
        WM_COMMAND if (wp.0 >> 16) as u32 == EN_KILLFOCUS => finish(h, true),
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}
