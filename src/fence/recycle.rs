use super::{arrange, bind, drop, fence_of, ghost, items, tabs};
use crate::{
    app::{change, with},
    shell,
    win::{cursor_pos, desktop_at},
};
use windows::Win32::{Foundation::HWND, System::Com::IDataObject};

fn place(tab: Option<u64>) {
    change(|c| c.place_recycle(tab));
    shell::recycle::show_on_desktop(tab.is_none());
    items::windows().into_iter().for_each(bind);
}

pub(super) fn toggle(h: HWND) {
    if let Some(t) = fence_of(h).map(|f| f.active().clone()) {
        place((!t.recycle).then_some(t.id));
    }
}

pub(super) fn drag(h: HWND, at: (i32, i32), content: (i32, i32)) {
    with(|a| a.recycle_drag = true);
    let image = items::index_at(h, at).and_then(|i| ghost::image(h, i, content));
    shell::drag_out(h, &[shell::recycle::path()], image, true);
    if with(|a| std::mem::take(&mut a.recycle_drag)) == Some(true) && desktop_at(cursor_pos()) {
        place(None);
    }
}

pub(super) fn dropped(h: HWND, data: Option<&IDataObject>, pt: (i32, i32)) -> bool {
    let ours = with(|a| std::mem::take(&mut a.recycle_drag)) == Some(true);
    if !ours && !data.is_some_and(shell::recycle::in_data) {
        return false;
    }
    match drop::target_tab(h, pt) {
        Some(tab) if tab.recycle && tabs::at(h, pt).is_none() => arrange::move_within(h, &[shell::recycle::path()], items::item_at(h, pt)),
        Some(tab) => place(Some(tab.id)),
        None => {}
    }
    true
}
