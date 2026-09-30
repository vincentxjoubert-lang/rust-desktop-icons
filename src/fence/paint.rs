use super::{TITLE, cell, header, label_font, metrics, title_font};
use crate::{
    app::with,
    domain::{anim, color, grid},
    layered::{ACCENT, Dib, Frame, WHITE},
    render::{flat, opaque_if_flat, premul, tint},
    win::*,
};
use windows::Win32::{Foundation::*, UI::WindowsAndMessaging::*};

pub fn render(h: HWND) {
    let wr = window_rect(h);
    let (w, ht) = (wr.right - wr.left, wr.bottom - wr.top);
    let s = |v| scale(h, v);
    let ((cell, icon), t, top_gap) = (metrics(h), s(TITLE), s(4));
    let (Some(mut frame), Some(mut icons)) = (Frame::new(w, ht), Dib::new(w, ht)) else {
        return;
    };
    with(|a| unsafe {
        let (ft, fi) = (title_font(a, h), label_font(a, h));
        let f = a.fence_of(h)?.clone();
        let v = a.view(h)?;
        let rolled = v.unroll == 0.;
        let body_h = if v.unroll < 1. { f.h } else { ht };
        header::text(&frame, ft, &f, t, s(8));
        let max = if rolled { 0 } else { grid::max_scroll(v.items.len(), w, body_h - t - top_gap, cell) };
        v.scroll = v.scroll.clamp(0, max);
        let cells: Vec<(usize, i32, i32)> = (0..if rolled { 0 } else { v.items.len() })
            .map(|i| (i, grid::origin(i, w, cell)))
            .map(|(i, (x, y))| (i, x, y + t + top_gap - v.scroll))
            .filter(|&(_, _, y)| y + cell > t && y < ht)
            .collect();
        frame.clip((0, t, w, ht));
        for &(i, x, y) in &cells {
            frame.text_w(fi, &mut v.items[i].name, &mut cell::label_at((x, y), cell, icon, s), cell::LABEL);
        }
        frame.target(&icons);
        for &(i, x, y) in &cells {
            let (ix, iy) = cell::icon_at((x, y), cell, icon, s);
            let _ = DrawIconEx(frame.dc(), ix, iy, v.items[i].icon, icon, icon, 0, None, DI_NORMAL);
        }
        frame.flush();
        for &(_, x, y) in &cells {
            let (ix, iy) = cell::icon_at((x, y), cell, icon, s);
            opaque_if_flat(icons.px(), w, (ix, iy, ix + icon, iy + icon));
        }
        if let Some(c) = f.look.tint {
            tint(icons.px(), c);
        }
        let (body, fg) = (f.look.alpha, color::contrast(f.look.color));
        let shade = if fg == WHITE || fg > 0x80_8080 { 0 } else { WHITE };
        let (full, rad, rows) = ((0, 0, w, ht), s(8) as f32, (0, ht));
        let tone = |k, a: u8| premul(color::shade(f.look.color, k), a);
        let mut c = frame.canvas();
        c.rrect(full, rad, (tone(10, body), tone(-14, body.saturating_add(20))), (t, ht), false);
        header::shapes(&mut c, &f, t, s);
        for &(i, x, y) in &cells {
            let (sel, hot) = (v.selected.contains(&v.items[i].path), v.hover == Some(i));
            if sel || hot {
                let r = (x + s(5), y + s(3), x + cell - s(5), y + cell - s(1));
                let (fill, line) = if sel { (premul(ACCENT, 90), premul(ACCENT, 200)) } else { (premul(WHITE, 40), premul(WHITE, 60)) };
                c.rrect(r, rad, flat(fill), (t, ht), false);
                c.rrect(r, rad, flat(line), (t, ht), true);
            }
        }
        if let Some(((x0, y0), (x1, y1))) = v.band {
            let off = t + top_gap - v.scroll;
            let r = (x0.min(x1), y0.min(y1) + off, x0.max(x1), y0.max(y1) + off);
            c.rrect(r, s(2) as f32, flat(premul(ACCENT, 50)), (t, ht), false);
            c.rrect(r, s(2) as f32, flat(premul(ACCENT, 200)), (t, ht), true);
        }
        c.layer(icons.px(), (0, 0), None, (t, ht));
        if max > 0 {
            let avail = ht - t;
            let th = (avail * avail / (avail + max)).max(s(24));
            let ty = t + v.scroll * (avail - th) / max;
            c.rrect((w - s(7), ty + s(3), w - s(3), ty + th - s(3)), s(2) as f32, flat(premul(WHITE, 110)), (t, ht), false);
        }
        frame.text_layers(fg, shade, s(2), rows);
        frame.canvas().rrect(full, rad, flat(premul(WHITE, 48)), rows, true);
        #[cfg(debug_assertions)]
        frame.dump(&f.id.to_string());
        frame.present(h, (wr.left, wr.top), anim::opacity(v.glow));
        Some(())
    });
}
