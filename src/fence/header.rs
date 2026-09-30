use crate::{
    domain::{Fence, color},
    layered::{Frame, WHITE},
    render::{Canvas, flat, premul},
};
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{DT_CENTER, DT_END_ELLIPSIS, DT_SINGLELINE, DT_VCENTER, HFONT},
};

fn segment(w: i32, n: usize, i: usize) -> (i32, i32) {
    let n = n.max(1) as i32;
    (i as i32 * w / n, (i as i32 + 1) * w / n)
}

pub(super) fn text(frame: &Frame, font: HFONT, f: &Fence, t: i32, pad: i32) {
    let (w, n) = (frame.w, f.tabs.len());
    let flags = DT_CENTER | DT_VCENTER | DT_SINGLELINE | DT_END_ELLIPSIS;
    if n == 1 {
        return frame.text(font, &f.tabs[0].title, RECT { left: t / 2, top: 0, right: w - t / 2, bottom: t }, flags);
    }
    for (i, tab) in f.tabs.iter().enumerate() {
        let (x0, x1) = segment(w, n, i);
        frame.text(font, &tab.title, RECT { left: x0 + pad, top: 0, right: x1 - pad, bottom: t }, flags);
    }
}

pub(super) fn shapes(c: &mut Canvas, f: &Fence, active: Option<usize>, t: i32, s: impl Fn(i32) -> i32) {
    let (w, n, body) = (c.w, f.tabs.len(), f.look.alpha);
    let tone = |k, a: u8| premul(color::shade(f.look.color, k), a);
    c.rrect((0, 0, w, t + s(16)), s(8) as f32, (tone(-20, body.saturating_add(50)), tone(-34, body.saturating_add(60))), (0, t), false);
    c.rrect((s(12), t - 1, w - s(12), t), 0., flat(premul(WHITE, 34)), (0, c.h), false);
    if n > 1 {
        if let Some(i) = active {
            let (x0, x1) = segment(w, n, i.min(n - 1));
            c.rrect((x0 + s(4), s(4), x1 - s(4), t - s(4)), s(6) as f32, flat(premul(WHITE, 38)), (0, t), false);
        }
        for i in 1..n {
            let x = segment(w, n, i).0;
            c.rrect((x, s(9), x + 1, t - s(9)), 0., flat(premul(WHITE, 40)), (0, t), false);
        }
    }
}
