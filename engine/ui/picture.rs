use core::{
    error::Error,
    ffi::{c_int, CStr},
    fmt,
};

use crate::{
    color::RGBA,
    engine,
    raw::{PictureFlags, HIMAGE},
    Size,
};

#[derive(Debug)]
pub enum PictureError {
    LoadError,
    InvalidPathError,
}

impl fmt::Display for PictureError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::LoadError => "failed to load a picture".fmt(f),
            Self::InvalidPathError => "invalid picture path".fmt(f),
        }
    }
}

impl Error for PictureError {}

pub struct Picture<T: AsRef<CStr> = &'static CStr> {
    raw: HIMAGE,
    path: T,
}

impl<T: AsRef<CStr>> Picture<T> {
    fn new(path: T, buf: Option<&[u8]>, flags: PictureFlags) -> Result<Self, PictureError> {
        engine()
            .pic_load(path.as_ref(), buf, flags.bits())
            .ok_or(PictureError::LoadError)
            .map(|raw| Self { raw, path })
    }

    pub fn raw(&self) -> HIMAGE {
        self.raw
    }

    pub fn path(&self) -> &T {
        &self.path
    }

    pub fn create_with_flags(
        path: T,
        buf: &[u8],
        flags: PictureFlags,
    ) -> Result<Self, PictureError> {
        if path.as_ref().to_bytes().starts_with(b"#") {
            Self::new(path, Some(buf), flags)
        } else {
            Err(PictureError::InvalidPathError)
        }
    }

    pub fn create(path: T, buf: &[u8]) -> Result<Self, PictureError> {
        Self::create_with_flags(path, buf, PictureFlags::empty())
    }

    pub fn load(path: T, flags: PictureFlags) -> Result<Self, PictureError> {
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
