use super::{cell, label_font, metrics};
use crate::{
    app::with,
    domain::grid,
    layered::{Dib, Frame, WHITE},
    render::{opaque_if_flat, straight},
    shell::{self, DragImage},
    win::*,
};
use windows::Win32::Foundation::HWND;

pub(super) fn image(h: HWND, index: usize, (cx, cy): (i32, i32)) -> Option<DragImage> {
    let s = |v| scale(h, v);
    let (cell, icon) = metrics(h);
    let (mut frame, mut icons) = (Frame::new(cell, cell)?, Dib::new(cell, cell)?);
    with(|a| {
        let font = label_font(a, h);
        let v = a.view(h)?;
        let item = v.items.get_mut(index)?;
        frame.text_w(font, &mut item.name, &mut cell::label_at((0, 0), cell, icon, s), cell::LABEL);
        frame.target(&icons);
        shell::draw_icon(frame.dc(), item.icon, cell::icon_at((0, 0), cell, icon, s), icon);
        Some(())
    })
    .flatten()?;
    frame.flush();
    let (ix, iy) = cell::icon_at((0, 0), cell, icon, s);
    opaque_if_flat(icons.px(), cell, (ix, iy, ix + icon, iy + icon));
    let px = icons.px().to_vec();
    frame.canvas().layer(&px, (0, 0), None, (0, cell));
    frame.text_layers(WHITE, 0, s(2), (0, cell));
    let (x, y) = grid::origin(index, client_rect(h).right, cell);
    let offset = (cx - x, cy - y);
    straight(frame.canvas.px());
    Some(DragImage { bitmap: frame.export_bitmap()?, size: (cell, cell), offset })
}
