use crate::{
    app::{Item, with},
    domain::{Fence, Zone, color, grid, zone},
    i18n::T,
    render::{Canvas, blur, flat, opaque_if_flat, premul},
    shell, store,
    win::*,
};
use std::{fs, path::PathBuf, ptr::null_mut, slice};
use windows::{
    Win32::{
        Foundation::*,
        Graphics::Gdi::*,
        UI::{
            Controls::{Dialogs::*, EM_LIMITTEXT, EM_SETSEL},
            Input::KeyboardAndMouse::{SetFocus, TME_LEAVE, TRACKMOUSEEVENT, TrackMouseEvent, VK_ESCAPE, VK_RETURN},
            Shell::*,
            WindowsAndMessaging::*,
        },
    },
    core::*,
};

pub const CLASS: PCWSTR = w!("RustDesktopIcons.Fence");
pub const WM_CHANGED: u32 = WM_APP + 10;
const TITLE: i32 = 34;
const CELL: i32 = 104;
const ICON: i32 = 48;
const BORDER: i32 = 6;
const EN_KILLFOCUS: u32 = 0x0200;
const WM_MOUSELEAVE: u32 = 0x02A3;
const WHITE: u32 = 0xFF_FFFF;

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

fn top() -> bool {
    cfg!(debug_assertions) && std::env::var_os("RDI_TOP").is_some()
}

pub fn create(f: &Fence) -> Option<HWND> {
    unsafe {
        let r = RECT { left: f.x, top: f.y, right: f.x + f.w, bottom: f.y + f.h };
        let (x, y) = if MonitorFromRect(&r, MONITOR_DEFAULTTONULL).is_invalid() { (100, 100) } else { (f.x, f.y) };
        let owner = if top() { None } else { FindWindowW(w!("Progman"), None).ok() };
        let ex = WS_EX_TOOLWINDOW | WS_EX_ACCEPTFILES | WS_EX_LAYERED;
        let h = CreateWindowExW(ex, CLASS, PCWSTR::null(), WS_POPUP, x, y, f.w, f.h, owner, None, Some(inst()), None).ok()?;
        if f.rolled {
            let _ = SetWindowPos(h, None, 0, 0, f.w, scale(h, TITLE), SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE);
        }
        if top() {
            let _ = SetWindowPos(h, Some(HWND_TOPMOST), 0, 0, 0, 0, SWP_NOMOVE | SWP_NOSIZE | SWP_NOACTIVATE);
        }
        let _ = ShowWindow(h, SW_SHOWNOACTIVATE);
        Some(h)
    }
}

fn apply(h: HWND) {
    let Some(f) = fence_of(h) else { return };
    let height = if f.rolled { scale(h, TITLE) } else { f.h };
    let _ = unsafe { SetWindowPos(h, None, 0, 0, f.w, height, SWP_NOMOVE | SWP_NOZORDER | SWP_NOACTIVATE) };
    render(h);
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
    render(h);
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
    let px = scale(h, 15);
    let Some((title, font)) = with(|a| {
        let id = a.view(h)?.id;
        let title = a.fence(id)?.title.clone();
        Some((title, a.font(px, FW_BOLD.0)))
    })
    .flatten() else {
        return;
    };
    let (r, t, pad) = (client(h), scale(h, TITLE), scale(h, 5));
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
    with(|a| a.view(h).map(|v| v.watch = None));
    if fs::remove_dir(&dir).is_err() && dir.exists() {
        let watch = shell::Watch::new(h, &dir, WM_CHANGED);
        with(|a| a.view(h).map(|v| v.watch = watch));
        return reload(h);
    }
    with(|a| {
        a.views.retain(|v| v.hwnd != h);
        a.cfg.fences.retain(|x| x.id != f.id);
        a.save();
    });
    unsafe { DestroyWindow(h).ok() };
}

fn index_at(h: HWND, (x, y): (i32, i32)) -> Option<usize> {
    let (w, t, cell) = (client(h).right, scale(h, TITLE), scale(h, CELL));
    with(|a| {
        let v = a.view(h)?;
        (y >= t).then(|| grid::index_at((x, y - t - scale(h, 4) + v.scroll), w, cell, v.items.len()))?
    })
    .flatten()
}

fn item_at(h: HWND, p: (i32, i32)) -> Option<PathBuf> {
    let i = index_at(h, p)?;
    with(|a| a.view(h)?.items.get(i).map(|it| it.path.clone())).flatten()
}

fn hover(h: HWND, i: Option<usize>) {
    if with(|a| a.view(h).map(|v| std::mem::replace(&mut v.hover, i) != i)).flatten() == Some(true) {
        render(h);
    }
}

fn context(h: HWND) {
    let mut p = POINT::default();
    unsafe {
        let _ = GetCursorPos(&mut p);
        let _ = ScreenToClient(h, &mut p);
    }
    if p.y < scale(h, TITLE) {
        return rename(h);
    }
    let target = item_at(h, (p.x, p.y));
    let Some(f) = fence_of(h) else { return };
    let Some((m, rtl)) = with(|a| {
        let m = menu();
        if target.is_some() {
            item(m, 30, a.t(T::Open), false);
            item(m, 31, a.t(T::Restore), false);
            item(m, 32, a.t(T::Delete), false);
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
        13 => shell::open(&store::fence_dir(f.id)),
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
        32 if target.is_some_and(|p| shell::recycle(&p)) => reload(h),
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
    let Some(id) = id(h) else { return };
    let (dir, fences, desks) = (store::fence_dir(id), store::root().join("fences"), shell::desktops());
    let is = |q: Option<&std::path::Path>, d: &std::path::Path| q.is_some_and(|q| shell::same_path(q, d));
    for p in paths.iter().filter(|p| !is(p.parent(), &dir)) {
        let own = desks.first().is_some_and(|d| is(p.parent(), d)) || is(p.parent().and_then(|q| q.parent()), &fences);
        let public = desks.get(1).is_some_and(|d| is(p.parent(), d));
        if own {
            let _ = store::move_into(p, &dir);
        } else if !(public && store::move_into(p, &dir).is_ok()) {
            let _ = shell::link_into(p, &dir);
        }
    }
    with(|a| a.views.iter().map(|v| v.hwnd).collect::<Vec<_>>()).into_iter().flatten().for_each(reload);
}

struct Dib {
    bmp: HBITMAP,
    ptr: *mut u32,
    len: usize,
}

impl Dib {
    fn new(w: i32, h: i32) -> Option<Self> {
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = null_mut();
        let bmp = unsafe { CreateDIBSection(None, &bi, DIB_RGB_COLORS, &mut bits, None, 0) }.ok()?;
        (!bits.is_null()).then(|| Self { bmp, ptr: bits.cast(), len: (w * h) as usize })
    }

    fn px(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        let _ = unsafe { DeleteObject(self.bmp.into()) };
    }
}

pub fn render(h: HWND) {
    let mut wr = RECT::default();
    unsafe { GetWindowRect(h, &mut wr).ok() };
    let (w, ht) = (wr.right - wr.left, wr.bottom - wr.top);
    let s = |v| scale(h, v);
    let (t, cell, icon, pad, top_gap) = (s(TITLE), s(CELL), s(ICON), s(10), s(4));
    let (Some(mut canvas), Some(mut icons), Some(mut mask)) = (Dib::new(w, ht), Dib::new(w, ht), Dib::new(w, ht)) else {
        return;
    };
    with(|a| unsafe {
        let (ft, fi) = (a.font(s(15), FW_BOLD.0), a.font(s(13), FW_SEMIBOLD.0));
        let id = a.view(h)?.id;
        let f = a.fence(id)?.clone();
        let v = a.view(h)?;
        let dc = CreateCompatibleDC(None);
        let old = SelectObject(dc, mask.bmp.into());
        SetBkMode(dc, TRANSPARENT);
        SetTextColor(dc, COLORREF(WHITE));
        SelectObject(dc, ft.into());
        let mut title: Vec<u16> = f.title.encode_utf16().collect();
        let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS | DT_NOPREFIX;
        DrawTextW(dc, &mut title, &mut RECT { left: t / 2, top: 0, right: w - t / 2, bottom: t }, flags);
        let max = if f.rolled { 0 } else { grid::max_scroll(v.items.len(), w, ht - t - top_gap, cell) };
        v.scroll = v.scroll.clamp(0, max);
        let cells: Vec<(usize, i32, i32)> = (0..if f.rolled { 0 } else { v.items.len() })
            .map(|i| (i, grid::origin(i, w, cell)))
            .map(|(i, (x, y))| (i, x, y + t + top_gap - v.scroll))
            .filter(|&(_, _, y)| y + cell > t && y < ht)
            .collect();
        SelectObject(dc, fi.into());
        IntersectClipRect(dc, 0, t, w, ht);
        for &(i, x, y) in &cells {
            let mut r = RECT { left: x + s(5), top: y + pad + icon + s(6), right: x + cell - s(5), bottom: y + cell };
            DrawTextW(dc, &mut v.items[i].name, &mut r, DT_CENTER | DT_WORDBREAK | DT_END_ELLIPSIS | DT_NOPREFIX | DT_EDITCONTROL);
        }
        SelectObject(dc, icons.bmp.into());
        let ix = |x: i32| x + (cell - icon) / 2;
        for &(i, x, y) in &cells {
            let _ = DrawIconEx(dc, ix(x), y + pad, v.items[i].icon, icon, icon, 0, None, DI_NORMAL);
        }
        let _ = GdiFlush();
        for &(_, x, y) in &cells {
            opaque_if_flat(icons.px(), w, (ix(x), y + pad, ix(x) + icon, y + pad + icon));
        }
        let body = f.alpha;
        let fg = color::contrast(f.color);
        let shade = if fg == WHITE || fg > 0x80_8080 { 0 } else { WHITE };
        let (full, rad, rows) = ((0, 0, w, ht), s(8) as f32, (0, ht));
        let soft = blur(mask.px(), w, ht, s(2));
        let mut c = Canvas { px: canvas.px(), w, h: ht };
        c.rrect(
            full,
            rad,
            (premul(color::shade(f.color, 10), body), premul(color::shade(f.color, -14), body.saturating_add(20))),
            (t, ht),
            false,
        );
        c.rrect(
            (0, 0, w, t + s(16)),
            rad,
            (premul(color::shade(f.color, -20), body.saturating_add(50)), premul(color::shade(f.color, -34), body.saturating_add(60))),
            (0, t),
            false,
        );
        c.rrect((s(12), t - 1, w - s(12), t), 0., flat(premul(WHITE, 34)), rows, false);
        if let Some(&(_, x, y)) = cells.iter().find(|c| Some(c.0) == v.hover) {
            let r = (x + s(5), y + s(3), x + cell - s(5), y + cell - s(1));
            c.rrect(r, s(8) as f32, flat(premul(WHITE, 40)), (t, ht), false);
            c.rrect(r, s(8) as f32, flat(premul(WHITE, 60)), (t, ht), true);
        }
        c.layer(icons.px(), (0, 0), None, (t, ht));
        c.layer(&soft, (0, s(1)), Some(premul(shade, 255)), rows);
        c.layer(&soft, (0, s(1)), Some(premul(shade, 200)), rows);
        c.layer(mask.px(), (0, s(1)), Some(premul(shade, 150)), rows);
        c.layer(mask.px(), (0, 0), Some(premul(fg, 255)), rows);
        if max > 0 {
            let avail = ht - t;
            let th = (avail * avail / (avail + max)).max(s(24));
            let ty = t + v.scroll * (avail - th) / max;
            c.rrect((w - s(7), ty + s(3), w - s(3), ty + th - s(3)), s(2) as f32, flat(premul(WHITE, 110)), (t, ht), false);
        }
        c.rrect(full, rad, flat(premul(WHITE, 48)), rows, true);
        #[cfg(debug_assertions)]
        if let Some(p) = std::env::var_os("RDI_DUMP") {
            let head = [w.to_le_bytes(), ht.to_le_bytes()].concat();
            let _ = fs::write(
                PathBuf::from(p).join(format!("{id}.bin")),
                [head, canvas.px().iter().flat_map(|v| v.to_le_bytes()).collect()].concat(),
            );
        }
        SelectObject(dc, canvas.bmp.into());
        let blend = BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: 255, AlphaFormat: AC_SRC_ALPHA as u8 };
        let (pos, size) = (POINT { x: wr.left, y: wr.top }, SIZE { cx: w, cy: ht });
        let _ =
            UpdateLayeredWindow(h, None, Some(&pos), Some(&size), Some(dc), Some(&POINT::default()), COLORREF(0), Some(&blend), ULW_ALPHA);
        SelectObject(dc, old);
        let _ = DeleteDC(dc);
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
        WM_MOUSEACTIVATE => return LRESULT(MA_NOACTIVATE as isize),
        WM_SIZE => render(h),
        WM_WINDOWPOSCHANGING => {
            let p = unsafe { &mut *(lp.0 as *mut WINDOWPOS) };
            if (p.flags & SWP_NOZORDER).0 == 0 && !top() {
                p.hwndInsertAfter = HWND_BOTTOM;
            }
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
        WM_NCRBUTTONUP if wp.0 as u32 == HTCAPTION => rename(h),
        WM_LBUTTONDBLCLK => item_at(h, xy(lp)).iter().for_each(|p| shell::open(p)),
        WM_CONTEXTMENU => context(h),
        WM_MOUSEMOVE => {
            let mut tme = TRACKMOUSEEVENT { cbSize: size_of::<TRACKMOUSEEVENT>() as u32, dwFlags: TME_LEAVE, hwndTrack: h, dwHoverTime: 0 };
            let _ = unsafe { TrackMouseEvent(&mut tme) };
            hover(h, index_at(h, xy(lp)));
        }
        WM_MOUSELEAVE => hover(h, None),
        WM_MOUSEWHEEL => {
            let step = (wp.0 >> 16) as i16 as i32 * scale(h, CELL) / 120;
            with(|a| a.view(h).map(|v| v.scroll -= step));
            render(h);
        }
        WM_DROPFILES => drop_files(h, HDROP(wp.0 as _)),
        WM_CHANGED => {
            unsafe { SetTimer(Some(h), 1, 150, None) };
        }
        WM_TIMER if wp.0 == 1 => {
            let _ = unsafe { KillTimer(Some(h), 1) };
            reload(h);
        }
        WM_COMMAND if (wp.0 >> 16) as u32 == EN_KILLFOCUS => finish(h, true),
        _ => return unsafe { DefWindowProcW(h, m, wp, lp) },
    }
    LRESULT(0)
}
