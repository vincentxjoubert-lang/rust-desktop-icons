use super::{close_rect, rows};
use crate::{
    app::with,
    layered::{Frame, WHITE},
    render::{flat, premul},
    win::*,
};
use windows::Win32::{Foundation::*, Graphics::Gdi::*};

const BG: u32 = 0x24_2220;
const ACCENT: u32 = 0xD4_7800;

pub(super) fn render(h: HWND) {
    let wr = window_rect(h);
    let (w, ht) = (wr.right - wr.left, wr.bottom - wr.top);
    let s = |v| scale(h, v);
    let Some(mut frame) = Frame::new(w, ht) else { return };
    let close = close_rect(h);
    with(|a| {
        let (title, bold, normal) = (a.font(s(17), FW_SEMIBOLD.0), a.font(s(12), FW_SEMIBOLD.0), a.font(s(13), FW_NORMAL.0));
        let rows = rows::build(a);
        let p = a.panel.as_ref()?;
        let (top, pad) = (s(rows::TOP), s(20));
        let left = DT_LEFT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
        let right = DT_RIGHT | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
        frame.text(
            title,
            a.t(crate::i18n::T::Settings).trim_end_matches('…'),
            RECT { left: pad, top: 0, right: w - s(60), bottom: top },
            left,
        );
        frame.text(normal, "×", close, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
        frame.clip((0, top, w, ht));
        let place = |y: i32, hh: i32| (top + s(y) - p.scroll, s(hh));
        for (r, &(y, hh)) in rows.iter().zip(&rows::layout(&rows)) {
            let (y, hh) = place(y, hh);
            let rect = RECT { left: pad, top: y, right: w - pad, bottom: y + hh };
            match &r.ctl {
                rows::Ctl::Header => frame.text(bold, &r.label.to_uppercase(), RECT { top: y + s(10), ..rect }, left),
                rows::Ctl::Button => frame.text(normal, &r.label, rect, DT_CENTER | DT_VCENTER | DT_SINGLELINE),
                rows::Ctl::Value(v) => {
                    frame.text(normal, &r.label, RECT { right: w / 2, ..rect }, left);
                    frame.text(normal, &format!("{v}  ›"), RECT { left: w / 2, ..rect }, right);
                }
                _ => frame.text(normal, &r.label, RECT { right: w - s(80), ..rect }, left),
            }
        }
        frame.flush();
        let mut c = frame.canvas();
        c.rrect((0, 0, w, ht), s(10) as f32, (premul(BG, 250), premul(0x1A_1817, 250)), (0, ht), false);
        c.rrect((0, top - 1, w, top), 0., flat(premul(WHITE, 30)), (0, ht), false);
        if p.close_hot {
            c.rrect((close.left, close.top, close.right, close.bottom), s(6) as f32, flat(premul(0x2B_11E8, 220)), (0, top), false);
        }
        let body = (top, ht);
        for (i, (r, &(y, hh))) in rows.iter().zip(&rows::layout(&rows)).enumerate() {
            let (y, hh) = place(y, hh);
            let hot = p.hover == Some(i);
            let box_ = (s(10), y + s(3), w - s(10), y + hh - s(3));
            match &r.ctl {
                rows::Ctl::Header => {}
                rows::Ctl::Button => {
                    c.rrect(box_, s(6) as f32, flat(premul(WHITE, if hot { 46 } else { 22 })), body, false);
                    c.rrect(box_, s(6) as f32, flat(premul(WHITE, 60)), body, true);
                }
                ctl => {
                    if hot {
                        c.rrect(box_, s(6) as f32, flat(premul(WHITE, 22)), body, false);
                    }
                    let (cy, x1) = (y + hh / 2, w - pad);
                    match ctl {
                        rows::Ctl::Toggle(on) => {
                            let track = (x1 - s(40), cy - s(10), x1, cy + s(10));
                            c.rrect(track, s(10) as f32, flat(premul(if *on { ACCENT } else { 0x55_5555 }, 255)), body, false);
                            let kx = if *on { x1 - s(18) } else { x1 - s(38) };
                            c.rrect((kx, cy - s(8), kx + s(16), cy + s(8)), s(8) as f32, flat(premul(WHITE, 255)), body, false);
                        }
                        rows::Ctl::Swatch(col) => {
                            let sw = (x1 - s(40), cy - s(10), x1, cy + s(10));
                            match col {
                                Some(col) => c.rrect(sw, s(5) as f32, flat(premul(*col, 255)), body, false),
                                None => c.rrect(sw, s(5) as f32, flat(premul(WHITE, 90)), body, true),
                            }
                            c.rrect(sw, s(5) as f32, flat(premul(WHITE, 70)), body, true);
                        }
                        _ => {}
                    }
                }
            }
        }
        frame.text_layers(0xF0_F0F0, 0, s(1), (0, ht));
        frame.canvas().rrect((0, 0, w, ht), s(10) as f32, flat(premul(WHITE, 40)), (0, ht), true);
        #[cfg(debug_assertions)]
        frame.dump("settings");
        frame.present(h, (wr.left, wr.top), 255);
        Some(())
    });
}
