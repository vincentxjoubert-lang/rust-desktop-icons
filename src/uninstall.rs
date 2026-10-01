use crate::{
    i18n::{self, T},
    report, shell, store,
    win::msgbox,
};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONWARNING, MB_OK};

pub fn run() {
    let (cfg, _) = store::load();
    if cfg.has_recycle() {
        shell::recycle::show_on_desktop(true);
    }
    let Some(desk) = shell::desktop() else { return };
    let errors: Vec<String> = cfg
        .fences
        .iter()
        .flat_map(|f| f.tabs.iter().filter(|t| t.portal.is_none()))
        .flat_map(|t| store::evacuate(&store::tab_dir(t), &desk).1)
        .collect();
    if errors.is_empty() {
        let _ = std::fs::remove_dir_all(store::root());
        return;
    }
    let lang = i18n::resolve(cfg.lang.as_deref(), &shell::locale());
    let text = i18n::get(lang, T::ErrDelete).replace("{}", &errors.iter().take(10).cloned().collect::<Vec<_>>().join("\n"));
    report::log(&text);
    msgbox(None, &text, MB_OK | MB_ICONWARNING, i18n::rtl(lang));
}
