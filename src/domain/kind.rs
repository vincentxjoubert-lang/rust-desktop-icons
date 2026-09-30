use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum Kind {
    Apps,
    Images,
    Documents,
    Videos,
    Music,
    Archives,
    Folders,
}

impl Kind {
    pub const ALL: [Kind; 7] = [Kind::Apps, Kind::Images, Kind::Documents, Kind::Videos, Kind::Music, Kind::Archives, Kind::Folders];

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
            Kind::Folders => &[],
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

pub fn transient(ext: Option<&str>) -> bool {
    ext.is_some_and(|e| ["crdownload", "part", "partial", "tmp", "download", "opdownload"].iter().any(|t| e.eq_ignore_ascii_case(t)))
}

#[cfg(test)]
mod tests {
    use super::*;

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
