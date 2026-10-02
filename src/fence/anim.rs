use super::{ANIM, layout, paint, view};
use crate::domain::Fence;
use crate::{
    app::with,
    domain::anim,
    win::{now_ms, window_rect},
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) fn start(h: HWND) {
    let idle = view(h, |v| v.tick.replace(v.tick.unwrap_or_else(now_ms)).is_none());
    if idle == Some(true) {
        unsafe { SetTimer(Some(h), ANIM, 10, None) };
    }
}

pub(super) fn cached(h: HWND, f: &Fence) -> bool {
    let busy = view(h, |v| v.inside || v.tick.is_some()).unwrap_or(false);
    if f.rolled { busy } else { f.look.chameleon }
}

pub(super) fn animating(h: HWND) -> bool {
    view(h, |v| v.tick.is_some()) == Some(true)
}

pub(super) fn tick(h: HWND) {
    let t = now_ms();
    let Some((f, done, alpha)) = with(|a| {
        let ms = a.cfg.roll_ms;
        let f = a.fence_of(h)?.clone();
        let v = a.view(h)?;
        let dt = (t - v.tick?) as f32;
        if dt < 1. {
            return None;
        }
        let unroll = if !f.rolled || (v.inside && !v.hold) { 1. } else { 0. };
        let glow = if !f.look.chameleon || v.inside { 1. } else { 0. };
        let opening = unroll > v.unroll;
        if opening != v.opening && v.unroll > 0. && v.unroll < 1. {
            v.unroll = anim::retarget(v.unroll, v.opening, opening);
        }
        v.opening = opening;
        v.unroll = anim::advance(v.unroll, unroll, dt, ms);
        v.glow = anim::advance(v.glow, glow, dt, ms);
        let done = v.unroll == unroll && v.glow == glow;
        v.tick = (!done).then_some(t);
        Some((f, done, anim::opacity(v.glow)))
    })
    .flatten() else {
        return;
    };
    let full = (f.w, layout::full(h, &f));
    let height = layout::height(h, &f);
    if done {
        let _ = unsafe { KillTimer(Some(h), ANIM) };
    }
    let cached = with(|a| a.view(h)?.frame.take()).flatten().filter(|fr| (fr.w, fr.h) == full);
    if done && height != full.1 {
        return layout::apply(h);
    }
    let Some(frame) = cached.or_else(|| paint::draw(h, full, true).map(|(fr, _)| fr)) else {
        return;
    };
    let r = window_rect(h);
    layout::self_sized(h, || frame.present_top(h, (r.left, r.top), height, alpha));
    paint::store(h, Some(frame));
}
