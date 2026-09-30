use crate::{domain::Fence, shell};
use std::{collections::HashSet, path::PathBuf};
use windows::Win32::{
    Foundation::HWND,
    UI::Shell::IContextMenu,
    UI::WindowsAndMessaging::{DestroyIcon, HICON},
};

pub struct Item {
    pub path: PathBuf,
    pub name: Vec<u16>,
    pub icon: HICON,
}

impl Drop for Item {
    fn drop(&mut self) {
        if !self.icon.is_invalid() {
            unsafe { DestroyIcon(self.icon).ok() };
        }
    }
}

pub struct View {
    pub hwnd: HWND,
    pub id: u64,
    pub items: Vec<Item>,
    pub scroll: i32,
    pub edit: Option<(HWND, Option<PathBuf>)>,
    pub native: Option<IContextMenu>,
    pub hover: Option<usize>,
    pub inside: bool,
    pub hold: bool,
    pub unroll: f32,
    pub glow: f32,
    pub tick: Option<std::time::Instant>,
    pub watches: Vec<shell::Watch>,
    pub cache: Vec<(u64, Vec<Item>)>,
    pub selected: HashSet<PathBuf>,
    pub band: Option<((i32, i32), (i32, i32))>,
}

impl View {
    pub(super) fn new(hwnd: HWND, f: &Fence) -> Self {
        let unroll = if f.rolled { 0. } else { 1. };
        let glow = if f.look.chameleon { 0. } else { 1. };
        Self {
            hwnd,
            id: f.id,
            items: vec![],
            scroll: 0,
            edit: None,
            native: None,
            hover: None,
            inside: false,
            hold: false,
            unroll,
            glow,
            tick: None,
            watches: vec![],
            cache: vec![],
            selected: HashSet::new(),
            band: None,
        }
    }
}
