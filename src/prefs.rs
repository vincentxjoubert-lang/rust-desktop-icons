use crate::{
    app::{App, change, register_verb},
    i18n::{self, T},
    shell,
    tray::updates,
    win::*,
};
use windows::Win32::UI::WindowsAndMessaging::HMENU;

pub const LANG_BASE: usize = 100;

pub fn lang_menu(a: &App) -> HMENU {
    let l = menu();
    item(l, LANG_BASE, a.t(T::Auto), a.cfg.lang.is_none());
    for (i, (code, name)) in i18n::LANGS.iter().enumerate() {
        item(l, LANG_BASE + 1 + i, name, a.cfg.lang.as_deref() == Some(*code));
    }
    l
}

pub fn lang_name(a: &App) -> &'static str {
    a.cfg.lang.as_deref().and_then(i18n::index).map_or(a.t(T::Auto), |i| i18n::LANGS[i].1)
}

pub fn set_lang(id: usize) -> bool {
    if !(LANG_BASE..=LANG_BASE + i18n::LANGS.len()).contains(&id) {
        return false;
    }
    change(|c| c.lang = id.checked_sub(LANG_BASE + 1).map(|i| i18n::LANGS[i].0.to_string()));
    register_verb();
    true
}

pub fn toggle_autostart() {
    change(|c| {
        c.autostart ^= true;
        shell::set_autostart(c.autostart);
    });
}

pub fn toggle_auto_update() {
    change(|c| {
        c.auto_update ^= true;
        updates::enable(c.auto_update);
    });
}
