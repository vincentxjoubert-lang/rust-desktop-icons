use windows::Win32::{
    Foundation::RECT,
    Graphics::Gdi::{DRAW_TEXT_FORMAT, DT_CENTER, DT_EDITCONTROL, DT_END_ELLIPSIS, DT_WORDBREAK},
};

pub(super) const LABEL: DRAW_TEXT_FORMAT = DRAW_TEXT_FORMAT(DT_CENTER.0 | DT_WORDBREAK.0 | DT_END_ELLIPSIS.0 | DT_EDITCONTROL.0);

pub(super) fn icon_at((x, y): (i32, i32), cell: i32, icon: i32, s: impl Fn(i32) -> i32) -> (i32, i32) {
    (x + (cell - icon) / 2, y + s(10))
}

pub(super) fn label_at((x, y): (i32, i32), cell: i32, icon: i32, s: impl Fn(i32) -> i32) -> RECT {
    RECT { left: x + s(5), top: y + s(10) + icon + s(6), right: x + cell - s(5), bottom: y + cell }
}
