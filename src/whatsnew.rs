use crate::{app::change, win::msgbox};
use windows::Win32::UI::WindowsAndMessaging::{MB_ICONINFORMATION, MB_OK};

const NOTES: &str = include_str!("../.github/release-notes.md");
const VERSION: &str = env!("CARGO_PKG_VERSION");

fn plain(md: &str) -> String {
    md.lines()
        .map(|l| l.trim_start_matches('#').trim_start().replace("**", "").replace('`', ""))
        .map(|l| l.strip_prefix("- ").map_or(l.clone(), |r| format!("•  {r}")))
        .collect::<Vec<_>>()
        .join("\n")
        .trim()
        .to_string()
}

pub fn upgraded(previous: Option<&str>, existed: bool) -> bool {
    existed && previous != Some(VERSION)
}

pub fn show(existed: bool) {
    let mut shown = false;
    change(|c| {
        shown = upgraded(c.version.as_deref(), existed);
        c.version = Some(VERSION.into());
    });
    if shown {
        msgbox(None, &plain(NOTES), MB_OK | MB_ICONINFORMATION, false);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn notes() {
        assert_eq!(plain("## What's new\n\n### Fixes\n- **Safe** `x`"), "What's new\n\nFixes\n•  Safe x");
        assert!(upgraded(None, true) && upgraded(Some("0.1.0"), true));
        assert!(!upgraded(Some(VERSION), true) && !upgraded(None, false));
        assert!(!NOTES.trim().is_empty());
    }
}
