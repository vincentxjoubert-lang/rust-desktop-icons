use super::Frame;
use crate::render::premul;
use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{DT_CENTER, DT_SINGLELINE, DT_VCENTER, HBITMAP, HFONT},
};

pub const RENAME: char = '\u{E8AC}';
pub const COLOR: char = '\u{E790}';
pub const OPACITY: char = '\u{E706}';
pub const SIZE: char = '\u{E80A}';
pub const TINT: char = '\u{E771}';
pub const EYE: char = '\u{E890}';
pub const ROLL: char = '\u{E70E}';
pub const TAB: char = '\u{E78B}';
pub const PORTAL: char = '\u{E71B}';
pub const CLOSE: char = '\u{E711}';
pub const FOLDER: char = '\u{E8B7}';
pub const DELETE: char = '\u{E74D}';
pub const ADD: char = '\u{E710}';
pub const SETTINGS: char = '\u{E713}';
pub const GLOBE: char = '\u{E774}';
pub const POWER: char = '\u{E7E8}';
pub const SYNC: char = '\u{E895}';
pub const REFRESH: char = '\u{E72C}';
pub const INFO: char = '\u{E946}';
pub const SPEED: char = '\u{EC4A}';
pub const SORT: char = '\u{E8CB}';
pub const CHECK: char = '\u{E73E}';
pub const DOWNLOAD: char = '\u{E896}';

pub fn bitmap(font: HFONT, ch: char, px: i32, color: u32) -> Option<HBITMAP> {
    let mut f = Frame::new(px, px)?;
    f.text(font, &ch.to_string(), RECT { left: 0, top: 0, right: px, bottom: px }, DT_CENTER | DT_VCENTER | DT_SINGLELINE);
    f.flush();
    let mask = f.mask.px().to_vec();
    f.canvas().layer(&mask, (0, 0), Some(premul(color, 255)), (0, px));
    f.export_bitmap()
}
