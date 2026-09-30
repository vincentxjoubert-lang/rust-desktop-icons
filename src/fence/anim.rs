use super::{ANIM, layout, render};
use crate::{app::with, domain::anim};
use std::time::Instant;
use windows::Win32::{
    Foundation::HWND,
    UI::WindowsAndMessaging::{KillTimer, SetTimer},
};

pub(super) fn start(h: HWND) {
    let idle = with(|a| a.view(h).map(|v| v.tick.replace(v.tick.unwrap_or_else(Instant::now)).is_none())).flatten();
    if idle == Some(true) {
        unsafe { SetTimer(Some(h), ANIM, 15, None) };
    }
}

pub(super) fn tick(h: HWND) {
    let Some((f, moved, done)) = with(|a| {
        let ms = a.cfg.roll_ms;
        let f = a.fence_of(h)?.clone();
        let v = a.view(h)?;
        let unroll = if !f.rolled || (v.inside && !v.hold) { 1. } else { 0. };
        let glow = if !f.look.chameleon || v.inside { 1. } else { 0. };
        let now = Instant::now();
        let dt = v.tick.map_or(0., |t| (now - t).as_secs_f32() * 1000.);
        let before = v.unroll;
        v.unroll = anim::advance(v.unroll, unroll, dt, ms);
        v.glow = anim::advance(v.glow, glow, dt, ms);
        let done = v.unroll == unroll && v.glow == glow;
        v.tick = (!done).then_some(now);
        Some((f, before != v.unroll, done))
    })
    .flatten() else {
        return;
    };
    if done {
        let _ = unsafe { KillTimer(Some(h), ANIM) };
    }
    layout::resize(h, &f);
    if !moved || done {
        render(h);
    }
}
