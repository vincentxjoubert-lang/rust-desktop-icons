use crate::{domain::Fence, layered::Frame, shell};
use std::{collections::HashSet, path::PathBuf};
use windows::Win32::{Foundation::HWND, UI::Shell::IContextMenu};

pub struct Item {
    pub path: PathBuf,
    pub name: Vec<u16>,
    pub icon: i32,
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
    pub tick: Option<f64>,
    pub frame: Option<Frame>,
    pub watches: Vec<shell::Watch>,
    pub cache: Vec<(u64, Vec<Item>)>,
    pub gens: Vec<(u64, u64)>,
    pub dirty: Vec<usize>,
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
            frame: None,
            watches: vec![],
            cache: vec![],
            gens: vec![],
            dirty: vec![],
            selected: HashSet::new(),
            band: None,
        }
    }
}
