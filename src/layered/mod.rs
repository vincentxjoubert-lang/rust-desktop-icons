mod dib;

use crate::render::{Canvas, blur, gray, premul};
pub use dib::Dib;
use windows::Win32::{Foundation::*, Graphics::Gdi::*, UI::WindowsAndMessaging::*};

pub const WHITE: u32 = 0xFF_FFFF;

pub struct Frame {
    pub w: i32,
    pub h: i32,
    pub canvas: Dib,
    pub mask: Dib,
    dc: HDC,
    old: HGDIOBJ,
}

impl Frame {
    pub fn new(w: i32, h: i32) -> Option<Self> {
        let (canvas, mask) = (Dib::new(w, h)?, Dib::new(w, h)?);
        unsafe {
            let dc = CreateCompatibleDC(None);
            let old = SelectObject(dc, mask.bmp.into());
            SetBkMode(dc, TRANSPARENT);
            SetTextColor(dc, COLORREF(WHITE));
            Some(Self { w, h, canvas, mask, dc, old })
        }
    }

    pub fn dc(&self) -> HDC {
        self.dc
    }

    pub fn text(&self, font: HFONT, text: &str, mut r: RECT, flags: DRAW_TEXT_FORMAT) {
        let mut t: Vec<u16> = text.encode_utf16().collect();
        self.text_w(font, &mut t, &mut r, flags);
    }

    pub fn text_w(&self, font: HFONT, text: &mut [u16], r: &mut RECT, flags: DRAW_TEXT_FORMAT) {
        unsafe {
            SelectObject(self.dc, font.into());
            DrawTextW(self.dc, text, r, flags | DT_NOPREFIX);
        }
    }

    pub fn clip(&self, (x0, y0, x1, y1): (i32, i32, i32, i32)) {
        unsafe { IntersectClipRect(self.dc, x0, y0, x1, y1) };
    }

    pub fn target(&self, dib: &Dib) {
        unsafe { SelectObject(self.dc, dib.bmp.into()) };
    }

    pub fn flush(&mut self) {
        let _ = unsafe { GdiFlush() };
        gray(self.mask.px());
    }

    pub fn canvas(&mut self) -> Canvas<'_> {
        Canvas { px: self.canvas.px(), w: self.w, h: self.h }
    }

    pub fn text_layers(&mut self, fg: u32, shadow: u32, r: i32, rows: (i32, i32)) {
        let (w, h) = (self.w, self.h);
        let soft = blur(self.mask.px(), w, h, r);
        let mask = self.mask.px().to_vec();
        let mut c = self.canvas();
        c.layer(&soft, (0, 1), Some(premul(shadow, 190)), rows);
        c.layer(&mask, (0, 0), Some(premul(fg, 255)), rows);
    }

    #[cfg(debug_assertions)]
    pub fn dump(&mut self, name: &str) {
        if let Some(p) = std::env::var_os("RDI_DUMP") {
            let head = [self.w.to_le_bytes(), self.h.to_le_bytes()].concat();
            let body = self.canvas.px().iter().flat_map(|v| v.to_le_bytes()).collect();
            let _ = std::fs::write(std::path::PathBuf::from(p).join(format!("{name}.bin")), [head, body].concat());
        }
    }

    pub fn present(&self, h: HWND, (x, y): (i32, i32), alpha: u8) {
        let blend =
            BLENDFUNCTION { BlendOp: AC_SRC_OVER as u8, BlendFlags: 0, SourceConstantAlpha: alpha, AlphaFormat: AC_SRC_ALPHA as u8 };
        let (pos, size) = (POINT { x, y }, SIZE { cx: self.w, cy: self.h });
        unsafe {
            SelectObject(self.dc, self.canvas.bmp.into());
            let _ = UpdateLayeredWindow(
                h,
                None,
                Some(&pos),
                Some(&size),
                Some(self.dc),
                Some(&POINT::default()),
                COLORREF(0),
                Some(&blend),
                ULW_ALPHA,
            );
        }
    }
}

impl Drop for Frame {
    fn drop(&mut self) {
        unsafe {
            SelectObject(self.dc, self.old);
            let _ = DeleteDC(self.dc);
        }
    }
}
