use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Games,
    Apps,
    Images,
    Documents,
    Videos,
    Music,
    Archives,
    Folders,
}

impl Kind {
    pub const ALL: [Kind; 8] =
        [Kind::Games, Kind::Apps, Kind::Images, Kind::Documents, Kind::Videos, Kind::Music, Kind::Archives, Kind::Folders];

    fn exts(self) -> &'static [&'static str] {
        match self {
            Kind::Apps => &["exe", "lnk", "url", "appref-ms", "bat", "cmd", "msi", "website"],
            Kind::Images => &["png", "jpg", "jpeg", "gif", "bmp", "webp", "svg", "ico", "tif", "tiff", "heic", "avif", "raw", "psd"],
            Kind::Documents => {
                &["pdf", "doc", "docx", "odt", "rtf", "txt", "md", "xls", "xlsx", "ods", "csv", "ppt", "pptx", "odp", "epub"]
            }
            Kind::Videos => &["mp4", "mkv", "avi", "mov", "wmv", "webm", "flv", "m4v", "mpg", "mpeg"],
            Kind::Music => &["mp3", "wav", "flac", "aac", "ogg", "m4a", "wma", "opus", "mid"],
            Kind::Archives => &["zip", "rar", "7z", "tar", "gz", "bz2", "xz", "iso", "cab"],
            Kind::Folders | Kind::Games => &[],
        }
    }

    pub fn of(ext: Option<&str>, dir: bool) -> Option<Kind> {
        if dir {
            return Some(Kind::Folders);
        }
        let ext = ext?.to_ascii_lowercase();
        Kind::ALL.into_iter().find(|k| k.exts().contains(&ext.as_str()))
    }
}

const GAME_MARKERS: [&str; 17] = [
    "steam://",
    r"\steamapps\common\",
    "com.epicgames.launcher://",
    r"\epic games\",
    r"\riot games\",
    "uplay://",
    r"\ubisoft game launcher\games\",
    "battlenet://",
    r"\battle.net\",
    "goggalaxy://",
    r"\gog galaxy\games\",
    r"\gog games\",
    "origin2://",
    r"\ea games\",
    r"\xboxgames\",
    r"\rockstar games\",
    r"\minecraft launcher\",
];

pub fn is_game(target: &str) -> bool {
    let t = target.to_lowercase();
    GAME_MARKERS.iter().any(|m| t.contains(m))
}

pub fn candidates(ext: Option<&str>, dir: bool, target: Option<&str>) -> Vec<Kind> {
    let base = Kind::of(ext, dir);
    if target.is_some_and(is_game) { [Some(Kind::Games), base].into_iter().flatten().collect() } else { base.into_iter().collect() }
}

pub fn transient(ext: Option<&str>) -> bool {
    ext.is_some_and(|e| ["crdownload", "part", "partial", "tmp", "download", "opdownload"].iter().any(|t| e.eq_ignore_ascii_case(t)))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn games() {
        assert!(is_game("steam://rungameid/252950"));
        assert!(is_game(r"D:\SteamLibrary\steamapps\common\Wreckfest\Wreckfest.exe"));
        assert!(is_game(r"C:\Program Files\Epic Games\Fortnite\Fortnite.exe"));
        assert!(is_game("com.epicgames.launcher://apps/Sugar?action=launch"));
        assert!(!is_game(r"C:\Program Files\Mozilla Firefox\firefox.exe"));
        assert_eq!(candidates(Some("url"), false, Some("steam://rungameid/1")), vec![Kind::Games, Kind::Apps]);
        assert_eq!(candidates(Some("lnk"), false, Some(r"C:\Windows\notepad.exe")), vec![Kind::Apps]);
        assert_eq!(candidates(Some("lnk"), false, None), vec![Kind::Apps]);
        assert_eq!(Kind::of(Some("exe"), false), Some(Kind::Apps));
    }

    #[test]
    fn kinds() {
        assert_eq!(Kind::of(Some("PNG"), false), Some(Kind::Images));
        assert_eq!(Kind::of(Some("lnk"), false), Some(Kind::Apps));
        assert_eq!(Kind::of(Some("docx"), false), Some(Kind::Documents));
        assert_eq!(Kind::of(None, true), Some(Kind::Folders));
        assert_eq!(Kind::of(Some("xyz"), false), None);
        assert_eq!(Kind::of(None, false), None);
        assert!(transient(Some("CRDOWNLOAD")) && !transient(Some("zip")) && !transient(None));
    }
}
