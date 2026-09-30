use super::{ANIM, layout, paint};
use crate::{
    app::with,
    domain::anim,
    win::{next_vblank_ms, now_ms, window_rect},
};
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) fn start(h: HWND) {
    let idle = with(|a| a.view(h).map(|v| v.tick.replace(v.tick.unwrap_or_else(now_ms)).is_none())).flatten();
    if idle == Some(true) {
        unsafe { SetTimer(Some(h), ANIM, 10, None) };
    }
}

pub(super) fn animating(h: HWND) -> bool {
    with(|a| a.view(h).map(|v| v.tick.is_some())).flatten() == Some(true)
}

pub(super) fn tick(h: HWND) {
    let t = next_vblank_ms().unwrap_or_else(now_ms);
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
        v.unroll = anim::advance(v.unroll, unroll, dt, ms);
        v.glow = anim::advance(v.glow, glow, dt, ms);
        let done = v.unroll == unroll && v.glow == glow;
        v.tick = (!done).then_some(t);
        if done {
            v.frame = None;
        }
        Some((f, done, anim::opacity(v.glow)))
    })
    .flatten() else {
        return;
    };
    if done {
        let _ = unsafe { KillTimer(Some(h), ANIM) };
        return layout::apply(h);
    }
    let full = (f.w, layout::full(h, &f));
    let cached = with(|a| a.view(h)?.frame.take()).flatten();
    let Some(frame) = cached.filter(|fr| (fr.w, fr.h) == full).or_else(|| paint::draw(h, full, true).map(|(fr, _)| fr)) else {
        return;
    };
    let r = window_rect(h);
    frame.present_top(h, (r.left, r.top), layout::height(h, &f), alpha);
    with(|a| a.view(h).map(|v| v.frame = Some(frame)));
}
