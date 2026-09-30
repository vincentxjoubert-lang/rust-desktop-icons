use std::{ptr::null_mut, slice};
use windows::Win32::Graphics::Gdi::*;

pub struct Dib {
    pub bmp: HBITMAP,
    ptr: *mut u32,
    len: usize,
}

impl Dib {
    pub fn new(w: i32, h: i32) -> Option<Self> {
        let bi = BITMAPINFO {
            bmiHeader: BITMAPINFOHEADER {
                biSize: size_of::<BITMAPINFOHEADER>() as u32,
                biWidth: w,
                biHeight: -h,
                biPlanes: 1,
                biBitCount: 32,
                ..Default::default()
            },
            ..Default::default()
        };
        let mut bits = null_mut();
        let bmp = unsafe { CreateDIBSection(None, &bi, DIB_RGB_COLORS, &mut bits, None, 0) }.ok()?;
        (!bits.is_null()).then(|| Self { bmp, ptr: bits.cast(), len: (w * h) as usize })
    }

    pub fn px(&mut self) -> &mut [u32] {
        unsafe { slice::from_raw_parts_mut(self.ptr, self.len) }
    }
}

impl Drop for Dib {
    fn drop(&mut self) {
        let _ = unsafe { DeleteObject(self.bmp.into()) };
    }
}
