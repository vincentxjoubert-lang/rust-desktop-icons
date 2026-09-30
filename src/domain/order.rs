use serde::{Deserialize, Serialize};
use std::cmp::Reverse;

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum Sort {
    #[default]
    Name,
    Type,
    Date,
    Custom,
}

impl Sort {
    pub const ALL: [Sort; 4] = [Sort::Name, Sort::Type, Sort::Date, Sort::Custom];
}

#[derive(Clone, Debug, PartialEq)]
pub struct Meta {
    pub name: String,
    pub dir: bool,
    pub modified: u64,
}

impl Meta {
    fn key(&self) -> String {
        self.name.to_lowercase()
    }

    fn ext(&self) -> String {
        self.name.rsplit_once('.').map(|(_, e)| e.to_lowercase()).unwrap_or_default()
    }
}

pub fn arrange(items: &[Meta], sort: Sort, custom: &[String]) -> Vec<usize> {
    let mut idx: Vec<usize> = (0..items.len()).collect();
    idx.sort_by_key(|&i| items[i].key());
    match sort {
        Sort::Name => {}
        Sort::Type => idx.sort_by_key(|&i| (!items[i].dir, items[i].ext())),
        Sort::Date => idx.sort_by_key(|&i| Reverse(items[i].modified)),
        Sort::Custom => idx.sort_by_key(|&i| custom.iter().position(|n| n.eq_ignore_ascii_case(&items[i].name)).unwrap_or(usize::MAX)),
    }
    idx
}

pub fn reorder(current: &[String], moved: &[String], before: Option<&str>) -> Vec<String> {
    let has = |list: &[String], n: &str| list.iter().any(|m| m.eq_ignore_ascii_case(n));
    let mut rest: Vec<String> = current.iter().filter(|n| !has(moved, n)).cloned().collect();
    let at = before.filter(|b| !has(moved, b)).and_then(|b| rest.iter().position(|n| n.eq_ignore_ascii_case(b))).unwrap_or(rest.len());
    rest.splice(at..at, moved.iter().cloned());
    rest
}

#[cfg(test)]
mod tests {
    use super::*;

    fn m(name: &str, dir: bool, modified: u64) -> Meta {
        Meta { name: name.into(), dir, modified }
    }

    #[test]
    fn sorting() {
        let items = [m("b.txt", false, 3), m("A.png", false, 1), m("zeta", true, 2), m("c.png", false, 5)];
        assert_eq!(arrange(&items, Sort::Name, &[]), vec![1, 0, 3, 2]);
        assert_eq!(arrange(&items, Sort::Type, &[]), vec![2, 1, 3, 0]);
        assert_eq!(arrange(&items, Sort::Date, &[]), vec![3, 0, 2, 1]);
        let custom = ["c.png".to_string(), "zeta".into()];
        assert_eq!(arrange(&items, Sort::Custom, &custom), vec![3, 2, 1, 0]);
    }

    #[test]
    fn reordering() {
        let cur: Vec<String> = ["a", "b", "c", "d"].map(String::from).to_vec();
        let mv = |m: &[&str], b| reorder(&cur, &m.iter().map(|s| s.to_string()).collect::<Vec<_>>(), b);
        assert_eq!(mv(&["d"], Some("b")), ["a", "d", "b", "c"]);
        assert_eq!(mv(&["a", "c"], Some("d")), ["b", "a", "c", "d"]);
        assert_eq!(mv(&["b"], None), ["a", "c", "d", "b"]);
        assert_eq!(mv(&["b"], Some("b")), ["a", "c", "d", "b"]);
        assert_eq!(mv(&["x"], Some("a")), ["x", "a", "b", "c", "d"]);
    }
}
