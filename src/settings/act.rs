use super::{refresh, rows};
use crate::{
    app::{change, rebuild, with},
    domain::{Kind, Look, anim, icons},
    i18n::{self, T},
    prefs, rules,
    tray::{self, updates},
    win::*,
};
use windows::Win32::Foundation::HWND;

fn choose(h: HWND, options: Vec<(String, bool)>) -> Option<usize> {
    let rtl = with(|a| a.rtl()).unwrap_or(false);
    let m = menu();
    for (i, (label, on)) in options.iter().enumerate() {
        item(m, i + 1, label, *on);
    }
    popup(h, m, rtl).checked_sub(1)
}

fn pick<V: Copy + PartialEq>(h: HWND, values: &[V], current: V, label: impl Fn(V) -> String) -> Option<V> {
    choose(h, values.iter().map(|&v| (label(v), v == current)).collect()).map(|i| values[i])
}

fn look() -> Look {
    with(|a| a.cfg.look.clone()).unwrap_or_default()
}

fn rule(h: HWND, fence: u64, tab: u64) {
    let Some(options) = with(|a| {
        let t = a.cfg.fences.iter().find(|f| f.id == fence)?.tabs.iter().find(|t| t.id == tab)?;
        Some(Kind::ALL.iter().map(|k| (a.t(i18n::kind(*k)).to_string(), t.kinds.contains(k))).collect())
    })
    .flatten() else {
        return;
    };
    let Some(k) = choose(h, options).map(|i| Kind::ALL[i]) else { return };
    change(|c| {
        if let Some(t) = c.fences.iter_mut().find(|f| f.id == fence).and_then(|f| f.tabs.iter_mut().find(|t| t.id == tab)) {
            if t.kinds.contains(&k) {
                t.kinds.retain(|x| *x != k);
            } else {
                t.kinds.push(k);
            }
        }
    });
}

fn perform(h: HWND, act: rows::Act) {
    use rows::Act::*;
    let l = look();
    match act {
        Color => {
            if let Some(c) = choose_color(h, l.color) {
                change(|cfg| cfg.look.color = c);
            }
        }
        Opacity => {
            if let Some(p) = pick(h, &Look::PERCENTS, l.percent(), Look::label) {
                change(|c| c.look.alpha = Look::alpha_for(p));
            }
        }
        IconSize => {
            if let Some(px) = pick(h, &icons::SIZES, l.icon, icons::label) {
                change(|c| c.look.icon = px);
            }
        }
        Tint => {
            let labels = with(|a| [a.t(T::None), a.t(T::Color)]).unwrap_or_default();
            match choose(h, vec![(labels[0].into(), l.tint.is_none()), (labels[1].into(), l.tint.is_some())]) {
                Some(0) => change(|c| c.look.tint = None),
                Some(_) => {
                    if let Some(t) = choose_color(h, l.tint.unwrap_or(0xFF_FFFF)) {
                        change(|c| c.look.tint = Some(t));
                    }
                }
                None => {}
            }
        }
        Chameleon => change(|c| c.look.chameleon ^= true),
        AutoHeight => change(|c| c.look.auto_height ^= true),
        Speed => {
            let current = with(|a| a.cfg.roll_ms).unwrap_or(anim::DEFAULT_MS);
            if let Some(ms) = pick(h, &anim::SPEEDS, current, |ms| with(|a| rows::speed_label(a, ms)).unwrap_or_default()) {
                change(|c| c.roll_ms = ms);
            }
        }
        ApplyAll => {
            change(|c| {
                let look = c.look.clone();
                c.fences.iter_mut().for_each(|f| f.look = look.clone());
            });
            rebuild();
        }
        AutoSort => change(|c| c.auto_sort ^= true),
        Rule(fence, tab) => rule(h, fence, tab),
        SortNow => rules::run(None, true),
        Lang => {
            let rtl = with(|a| a.rtl()).unwrap_or(false);
            if let Some(m) = with(|a| prefs::lang_menu(a)) {
                prefs::set_lang(popup(h, m, rtl));
            }
        }
        Autostart => prefs::toggle_autostart(),
        AutoUpdate => prefs::toggle_auto_update(),
        Check => {
            if let Some(t) = tray::hwnd() {
                updates::check_now(t);
            }
        }
    }
}

pub(super) fn click(h: HWND, i: usize) {
    if let Some(act) = with(|a| rows::build(a).get(i).and_then(|r| r.act)).flatten() {
        perform(h, act);
        refresh();
    }
}
