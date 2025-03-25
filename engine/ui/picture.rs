use core::ffi::{c_int, CStr};

use shared::{color::RGBA, raw::PictureFlags};

use crate::{engine, raw::HIMAGE, Size};

pub struct Picture<T: AsRef<CStr> = &'static CStr> {
    raw: HIMAGE,
    path: T,
}

impl<T: AsRef<CStr>> Picture<T> {
    fn new(path: T, buf: Option<&[u8]>, flags: PictureFlags) -> Self {
        let raw = engine().pic_load(path.as_ref(), buf, flags.bits());
        // TODO: return Result
        assert!(raw != 0);
        Self { raw, path }
    }

    pub fn raw(&self) -> HIMAGE {
        self.raw
    }

    pub fn path(&self) -> &T {
        &self.path
    }

    pub fn create_with_flags(path: T, buf: &[u8], flags: PictureFlags) -> Self {
        // TODO: return Result
        assert!(path.as_ref().to_bytes().starts_with(b"#"));
        Self::new(path, Some(buf), flags)
    }

    pub fn create(path: T, buf: &[u8]) -> Self {
        Self::create_with_flags(path, buf, PictureFlags::empty())
    }

    pub fn load(path: T, flags: PictureFlags) -> Self {
        Self::new(path, None, flags)
    }

    pub fn as_raw(&self) -> c_int {
        self.raw
    }

    pub fn width(&self) -> c_int {
        engine().pic_width(self.raw)
    }

    pub fn height(&self) -> c_int {
        engine().pic_height(self.raw)
    }

    pub fn size(&self) -> Size {
        engine().pic_size(self.raw)
    }

    pub fn set_with_color<C: Into<RGBA>>(&self, color: C) {
        engine().pic_set(self.raw, color);
    }

    pub fn set(&self) {
        self.set_with_color(RGBA::WHITE);
    }
}

// FIXME: The engine uses one id for all pictures with same path.
// impl<T: AsRef<CStr>> Drop for Picture<T> {
//     fn drop(&mut self) {
//         engine().pic_free(self.path.as_ref());
//     }
// }
