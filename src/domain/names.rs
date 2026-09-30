pub fn renamed(file: &str, shown: &str, input: &str) -> Option<String> {
    let input: String = input.trim().chars().filter(|c| !c.is_control() && !r#"\/:*?"<>|"#.contains(*c)).collect();
    let input = input.trim_end_matches(['.', ' ']);
    if input.is_empty() {
        return None;
    }
    let hidden = file.rsplit_once('.').filter(|_| file != shown).map(|(_, e)| e);
    let name = match hidden {
        Some(ext) if !input.to_lowercase().ends_with(&format!(".{}", ext.to_lowercase())) => format!("{input}.{ext}"),
        _ => input.to_string(),
    };
    (name != file).then_some(name)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn renaming() {
        assert_eq!(renamed("Jeu.lnk", "Jeu", "Super jeu").as_deref(), Some("Super jeu.lnk"));
        assert_eq!(renamed("Jeu.lnk", "Jeu", "Super.lnk").as_deref(), Some("Super.lnk"));
        assert_eq!(renamed("notes.txt", "notes.txt", "todo.md").as_deref(), Some("todo.md"));
        assert_eq!(renamed("a.txt", "a.txt", "  a.txt "), None);
        assert_eq!(renamed("a.txt", "a.txt", "b/c:d?.txt ").as_deref(), Some("bcd.txt"));
        assert_eq!(renamed("a.txt", "a.txt", "   "), None);
        assert_eq!(renamed("Dossier", "Dossier", "Projets").as_deref(), Some("Projets"));
    }
}
