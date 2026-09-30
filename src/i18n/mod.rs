use crate::domain::{Kind, Sort};

#[derive(Clone, Copy)]
pub enum T {
    NewFence,
    Rename,
    Color,
    Opacity,
    Roll,
    OpenFolder,
    DeleteFence,
    Language,
    Autostart,
    AutoUpdate,
    CheckUpdates,
    About,
    Quit,
    ConfirmDelete,
    UpToDate,
    Fence,
    UpdateFailed,
    Auto,
    IconSize,
    DesktopVerb,
    NewTab,
    NewPortal,
    DeleteTab,
    ConfirmDeleteTab,
    Tint,
    None,
    Chameleon,
    Settings,
    Appearance,
    ApplyAll,
    RollSpeed,
    Instant,
    Rules,
    AutoSort,
    SortNow,
    General,
    KApps,
    KImages,
    KDocuments,
    KVideos,
    KMusic,
    KArchives,
    KFolders,
    Behavior,
    SortBy,
    SortName,
    SortType,
    SortDate,
    SortCustom,
    AutoHeight,
    ErrConfig,
    ErrSave,
    ErrMove,
    ErrRename,
    ErrDelete,
    ErrInstall,
    Lock,
}

mod table;

use table::S;

const N: usize = 57;

pub const LANGS: [(&str, &str); 25] = [
    ("en", "English"),
    ("zh", "中文"),
    ("hi", "हिन्दी"),
    ("es", "Español"),
    ("fr", "Français"),
    ("ar", "العربية"),
    ("bn", "বাংলা"),
    ("pt", "Português"),
    ("ru", "Русский"),
    ("ur", "اردو"),
    ("id", "Bahasa Indonesia"),
    ("de", "Deutsch"),
    ("ja", "日本語"),
    ("sw", "Kiswahili"),
    ("mr", "मराठी"),
    ("te", "తెలుగు"),
    ("tr", "Türkçe"),
    ("ta", "தமிழ்"),
    ("vi", "Tiếng Việt"),
    ("ko", "한국어"),
    ("it", "Italiano"),
    ("th", "ไทย"),
    ("pl", "Polski"),
    ("uk", "Українська"),
    ("fa", "فارسی"),
];

pub fn kind(k: Kind) -> T {
    match k {
        Kind::Apps => T::KApps,
        Kind::Images => T::KImages,
        Kind::Documents => T::KDocuments,
        Kind::Videos => T::KVideos,
        Kind::Music => T::KMusic,
        Kind::Archives => T::KArchives,
        Kind::Folders => T::KFolders,
    }
}

pub fn sort(s: Sort) -> T {
    match s {
        Sort::Name => T::SortName,
        Sort::Type => T::SortType,
        Sort::Date => T::SortDate,
        Sort::Custom => T::SortCustom,
    }
}

pub fn get(lang: usize, key: T) -> &'static str {
    S[lang.min(S.len() - 1)][key as usize]
}

pub fn index(code: &str) -> Option<usize> {
    LANGS.iter().position(|(c, _)| code.eq_ignore_ascii_case(c))
}

pub fn resolve(pref: Option<&str>, system: &str) -> usize {
    pref.and_then(index).or_else(|| index(system.split(['-', '_']).next()?)).unwrap_or(0)
}

pub fn rtl(lang: usize) -> bool {
    matches!(LANGS[lang.min(LANGS.len() - 1)].0, "ar" | "ur" | "fa")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_tables() {
        assert!(S.iter().flatten().all(|s| !s.trim().is_empty()));
        assert_eq!(T::Lock as usize, N - 1);
    }

    #[test]
    fn resolution() {
        assert_eq!(resolve(None, "fr-FR"), 4);
        assert_eq!(resolve(Some("ja"), "fr-FR"), 12);
        assert_eq!(resolve(Some("xx"), "zh_CN"), 1);
        assert_eq!(resolve(None, ""), 0);
        assert!(rtl(5) && !rtl(0));
        assert_eq!(get(4, T::Quit), "Quitter");
    }
}
