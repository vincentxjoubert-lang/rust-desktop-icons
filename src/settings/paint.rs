use super::{close_rect, rows};
use crate::{
    app::with,
    i18n::T,
    layered::{ACCENT, Frame, WHITE, glyph as g},
    render::{Canvas, flat, premul},
    win::*,
};
use windows::Win32::{Foundation::*, Graphics::Gdi::*};

const BG: u32 = 0x24_2220;
const LINK: u32 = 0xFF_CD60;

fn toggle(c: &mut Canvas, (x1, cy): (i32, i32), on: bool, s: &impl Fn(i32) -> i32, clip: (i32, i32)) {
    let track = (x1 - s(40), cy - s(10), x1, cy + s(10));
    c.rrect(track, s(10) as f32, flat(premul(if on { ACCENT } else { 0x55_5555 }, 255)), clip, false);
    let kx = if on { x1 - s(18) } else { x1 - s(38) };
    c.rrect((kx, cy - s(8), kx + s(16), cy + s(8)), s(8) as f32, flat(premul(WHITE, 255)), clip, false);
}

fn swatch(c: &mut Canvas, (x1, cy): (i32, i32), col: Option<u32>, s: &impl Fn(i32) -> i32, clip: (i32, i32)) {
    let sw = (x1 - s(40), cy - s(11), x1, cy + s(11));
    if let Some(col) = col {
        c.rrect(sw, s(6) as f32, flat(premul(col, 255)), clip, false);
    }
    c.rrect(sw, s(6) as f32, flat(premul(WHITE, 80)), clip, true);
}

pub(super) fn render(h: HWND) {
    let wr = window_rect(h);
    let (w, ht) = (wr.right - wr.left, wr.bottom - wr.top);
    let s = |v| scale(h, v);
    let Some(mut frame) = Frame::new(w, ht) else { return };
    let close = close_rect(h);
    with(|a| {
        let (title, bold, normal, icons) =
            (a.font(s(18), FW_SEMIBOLD.0), a.font(s(12), FW_SEMIBOLD.0), a.font(s(13), FW_NORMAL.0), a.icon_font(s(15)));
        let rows = rows::build(a);
        let p = a.panel.as_ref()?;
        let (top, pad) = (s(rows::TOP), s(16));
        let (x_icon, x_label, x_end) = (pad + s(14), pad + s(46), w - pad - s(14));
        let left = DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
        let right = DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
        let center = DT_CENTER | DT_VCENTER | DT_SINGLELINE;
        let layout: Vec<(i32, i32)> = rows::layout(&rows).iter().map(|&(y, hh)| (top + s(y) - p.scroll, s(hh))).collect();
        let body = (top, ht);
        frame.text(title, a.t(T::Settings).trim_end_matches('…'), RECT { left: s(22), top: 0, right: w - s(60), bottom: top }, left);
        frame.text(icons, &g::CLOSE.to_string(), close, center);
        frame.clip((0, top, w, ht));
        for (r, &(y, hh)) in rows.iter().zip(&layout) {
            let rect = RECT { left: x_label, top: y, right: x_end, bottom: y + hh };
            match &r.ctl {
                rows::Ctl::Header => frame.text(bold, &r.label.to_uppercase(), RECT { left: pad + s(6), top: y + s(14), ..rect }, left),
                rows::Ctl::Button => {}
                rows::Ctl::Value(v) => {
                    frame.text(normal, &r.label, RECT { right: w / 2 + s(20), ..rect }, left);
                    frame.text(normal, &format!("{v}   ›"), RECT { left: w / 2 + s(20), ..rect }, right);
                }
                _ => frame.text(normal, &r.label, RECT { right: x_end - s(56), ..rect }, left),
            }
        }
        frame.flush();
        let mut c = frame.canvas();
        c.rrect((0, 0, w, ht), s(12) as f32, (premul(BG, 252), premul(0x1A_1817, 252)), (0, ht), false);
        c.rrect((0, top - 1, w, top), 0., flat(premul(WHITE, 26)), (0, ht), false);
        if p.close_hot {
            c.rrect((close.left, close.top, close.right, close.bottom), s(6) as f32, flat(premul(0x2B_11E8, 230)), (0, top), false);
        }
        for (first, last) in rows::cards(&rows) {
            let (y0, y1) = (layout[first].0, layout[last].0 + layout[last].1);
            c.rrect((pad, y0, w - pad, y1), s(8) as f32, flat(premul(WHITE, 12)), body, false);
            c.rrect((pad, y0, w - pad, y1), s(8) as f32, flat(premul(WHITE, 24)), body, true);
            for &(y, _) in &layout[first + 1..=last] {
                c.rrect((x_label, y, w - pad - s(1), y + 1), 0., flat(premul(WHITE, 18)), body, false);
            }
        }
        for (i, (r, &(y, hh))) in rows.iter().zip(&layout).enumerate() {
            if p.hover == Some(i) {
                c.rrect((pad + s(3), y + s(3), w - pad - s(3), y + hh - s(3)), s(6) as f32, flat(premul(WHITE, 20)), body, false);
            }
            let at = (x_end, y + hh / 2);
            match &r.ctl {
                rows::Ctl::Toggle(on) => toggle(&mut c, at, *on, &s, body),
                rows::Ctl::Swatch(col) => swatch(&mut c, at, *col, &s, body),
                _ => {}
            }
        }
        frame.text_layers(0xF2_F2F2, 0, s(1), (0, ht));
        frame.clear_mask();
        for (r, &(y, hh)) in rows.iter().zip(&layout) {
            if r.ctl == rows::Ctl::Header {
                continue;
            }
            frame.text(icons, &r.glyph.to_string(), RECT { left: x_icon, top: y, right: x_icon + s(22), bottom: y + hh }, center);
            if r.ctl == rows::Ctl::Button {
                frame.text(normal, &r.label, RECT { left: x_label, top: y, right: x_end, bottom: y + hh }, left);
            }
        }
        frame.flush();
        frame.text_layers(LINK, 0, s(1), body);
        frame.canvas().rrect((0, 0, w, ht), s(12) as f32, flat(premul(WHITE, 40)), (0, ht), true);
        #[cfg(debug_assertions)]
        frame.dump("settings");
        frame.present(h, (wr.left, wr.top), 255);
        Some(())
    });
}
