use core::{ffi::c_int, fmt, str::FromStr};

use crate::raw::color24;

macro_rules! define_colors {
    ($($value:expr => $name:ident),* $(,)?) => {
        impl RGB {
            $(pub const $name: Self = Self::from_u32_rgb($value);)*
        }

        impl RGBA {
            $(pub const $name: Self = Self::from_u32_argb($value);)*
        }
    };
}

define_colors! {
    0xff000000 => BLACK,
    0xffc0c0c0 => SILVER,
    0xff808080 => GRAY,
    0xffffffff => WHITE,
    0xff800000 => MAROON,
    0xffff0000 => RED,
    0xff00ff00 => GREEN,
    0xff00ff00 => LIME,
    0xff000080 => NAVY,
    0xff0000ff => BLUE,
    0xffffa000 => YELLOWISH,
    0xffff1010 => REDISH,
    0xff00a000 => GREENISH,
    0xff800080 => PURPLE,
    0xffff00ff => FUCHSIA,
    0xff00ffff => CYAN,
}

macro_rules! impl_get_set {
    ($($i:expr => $field:ident, $field_set:ident;)*) => {
        $(
            #[inline(always)]
            pub const fn $field(&self) -> u8 {
                let bytes = self.to_bytes();
                bytes[$i]
            }

            #[inline(always)]
            pub fn $field_set(&mut self, value: u8) {
                let mut bytes = self.to_bytes();
                bytes[$i] = value;
                *self = Self::from_bytes(bytes);
            }
        )*
    };
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct RGB(u32);

impl RGB {
    pub const fn from_bytes([r, g, b]: [u8; 3]) -> RGB {
        Self(u32::from_le_bytes([r, g, b, 0]))
    }

    pub const fn to_bytes(self) -> [u8; 3] {
        let [r, g, b, _] = self.0.to_le_bytes();
        [r, g, b]
    }

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self::from_bytes([r, g, b])
    }

    const fn from_u32_rgb(value: u32) -> Self {
        let [_, r, g, b] = value.to_be_bytes();
        Self::new(r, g, b)
    }

    pub const fn splat(c: u8) -> Self {
        Self::new(c, c, c)
    }

    pub const fn rgba(self, a: u8) -> RGBA {
        let [r, g, b] = self.to_bytes();
        RGBA::new(r, g, b, a)
    }

    impl_get_set! {
        0 => r, set_r;
        1 => g, set_g;
        2 => b, set_b;
    }

    pub fn saturating_add(self, other: impl Into<RGB>) -> RGB {
        let mut c = self.to_bytes();
        let o = other.into().to_bytes();
        for i in 0..3 {
            c[i] = c[i].saturating_add(o[i]);
        }
        Self::from_bytes(c)
    }

    pub fn scale_color(self, a: u8) -> Self {
        if a == 255 {
            return self;
        }
        let a = a as u16 + 1;
        let mut c = self.to_bytes();
        for i in &mut c {
            *i = ((*i as u16 * a) >> 8) as u8;
        }
        Self::from_bytes(c)
    }

    #[deprecated(note = "use scale_color instead")]
    pub fn scale(self, a: u8) -> Self {
        self.scale_color(a)
    }

    pub fn blend_alpha(self, other: impl Into<RGB>, alpha: u8) -> RGB {
        if alpha == 255 {
            return self;
        }
        let mut c = self.scale_color(alpha).to_bytes();
        let o = other.into().scale_color(255 - alpha).to_bytes();
        for i in 0..3 {
            c[i] += o[i];
        }
        Self::from_bytes(c)
    }

    pub const fn to_rgba(self) -> RGBA {
        let [r, g, b] = self.to_bytes();
        RGBA::rgb(r, g, b)
    }
}

impl From<[u8; 3]> for RGB {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::new(r, g, b)
    }
}

impl From<RGBA> for RGB {
    fn from(color: RGBA) -> RGB {
        color.to_rgb()
    }
}

impl From<RGB> for [u8; 3] {
    fn from(color: RGB) -> Self {
        color.to_bytes()
    }
}

impl From<RGB> for [c_int; 3] {
    fn from(color: RGB) -> Self {
        let [r, g, b] = color.to_bytes();
        [r as c_int, g as c_int, b as c_int]
    }
}

impl From<color24> for RGB {
    fn from(color: color24) -> RGB {
        RGB::new(color.r, color.g, color.b)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(transparent)]
pub struct RGBA(u32);

impl RGBA {
    pub const fn from_bytes([r, g, b, a]: [u8; 4]) -> RGBA {
        Self(u32::from_le_bytes([r, g, b, a]))
    }

    pub const fn to_bytes(self) -> [u8; 4] {
        self.0.to_le_bytes()
    }

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        Self::from_bytes([r, g, b, a])
    }

    const fn from_u32_argb(value: u32) -> RGBA {
        let [a, r, g, b] = value.to_be_bytes();
        Self::new(r, g, b, a)
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> RGBA {
        Self::new(r, g, b, 255)
    }

    pub const fn splat(c: u8) -> RGBA {
        Self::new(c, c, c, c)
    }

    pub const fn splat_color(c: u8) -> RGBA {
        Self::new(c, c, c, 255)
    }

    impl_get_set! {
        0 => r, set_r;
        1 => g, set_g;
        2 => b, set_b;
        3 => a, set_a;
    }

    pub const fn to_rgb(self) -> RGB {
        let [r, g, b, _] = self.to_bytes();
        RGB::new(r, g, b)
    }

    pub fn scale_color(self, a: u8) -> RGBA {
        if a == 255 {
            return self;
        }
        let a = a as u16 + 1;
        let mut c = self.to_bytes();
        for i in c[..3].iter_mut() {
            *i = ((*i as u16 * a) >> 8) as u8;
        }
        Self::from_bytes(c)
    }

    pub fn blend_color(self, color: impl Into<RGB>) -> RGBA {
        let mut c = self.to_bytes();
        let o = color.into().to_bytes();
        for i in 0..3 {
            c[i] = ((c[i] as u16 * o[i] as u16) >> 8) as u8;
        }
        Self::from_bytes(c)
    }

    pub fn blend_color_with_alpha(self, other: impl Into<RGB>, alpha: u8) -> RGBA {
        if alpha == 255 {
            return self;
        }
        let mut c = self.scale_color(alpha).to_bytes();
        let o = other.into().scale_color(255 - alpha).to_bytes();
        for i in 0..3 {
            c[i] += o[i];
        }
        Self::from_bytes(c)
    }

    pub fn blend(self, other: impl Into<RGBA>) -> Self {
        let mut c = self.to_bytes();
        let o = other.into().to_bytes();
        for i in 0..4 {
            c[i] = ((c[i] as u16 * o[i] as u16) >> 8) as u8;
        }
        Self::from_bytes(c)
    }
}

impl From<[u8; 3]> for RGBA {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::rgb(r, g, b)
    }
}

impl From<[u8; 4]> for RGBA {
    fn from([r, g, b, a]: [u8; 4]) -> Self {
        Self::new(r, g, b, a)
    }
}

impl From<RGB> for RGBA {
    fn from(color: RGB) -> RGBA {
        let [r, g, b] = color.to_bytes();
        RGBA::rgb(r, g, b)
    }
}

impl From<color24> for RGBA {
    fn from(color: color24) -> RGBA {
        RGBA::rgb(color.r, color.g, color.b)
    }
}

impl From<RGBA> for [u8; 3] {
    fn from(color: RGBA) -> Self {
        let [r, g, b, _] = color.to_bytes();
        [r, g, b]
    }
}

impl From<RGBA> for [u8; 4] {
    fn from(color: RGBA) -> Self {
        color.to_bytes()
    }
}

impl From<RGBA> for [c_int; 4] {
    fn from(color: RGBA) -> Self {
        let [r, g, b, a] = color.to_bytes();
        [r as c_int, g as c_int, b as c_int, a as c_int]
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct ParseColorError;

impl From<core::num::ParseIntError> for ParseColorError {
    fn from(_: core::num::ParseIntError) -> Self {
        ParseColorError
    }
}

impl FromStr for RGB {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if !s.starts_with("#") && s.len() != 7 {
            return Err(ParseColorError);
        }
        let r = u8::from_str_radix(&s[1..3], 16)?;
        let g = u8::from_str_radix(&s[3..5], 16)?;
        let b = u8::from_str_radix(&s[5..7], 16)?;
        Ok(Self::new(r, g, b))
    }
}

impl FromStr for RGBA {
    type Err = ParseColorError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if ![7, 9].contains(&s.len()) {
            return Err(ParseColorError);
        }
        let mut ret = RGB::from_str(&s[..7])?.to_rgba();
        if s.len() == 9 {
            ret.set_a(u8::from_str_radix(&s[7..9], 16)?);
        };
        Ok(ret)
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let [r, g, b] = self.to_bytes();
        write!(f, "#{r:02x}{g:02x}{b:02x}")
    }
}

impl fmt::Display for RGBA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let [r, g, b, a] = self.to_bytes();
        if a != u8::MAX {
            write!(f, "#{r:02x}{g:02x}{b:02x}{a:02x}")
        } else {
            write!(f, "#{r:02x}{g:02x}{b:02x}")
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rgb_parse() {
        assert_eq!("#010203".parse(), Ok(RGB::new(1, 2, 3)));
        assert_eq!("#ffffff".parse(), Ok(RGB::WHITE));
        assert_eq!("#ff0000".parse(), Ok(RGB::RED));
        assert_eq!("#00ff00".parse(), Ok(RGB::GREEN));
        assert_eq!("#0000ff".parse(), Ok(RGB::BLUE));
    }

    #[test]
    fn rgb_format() {
        assert_eq!("#010203", RGB::new(1, 2, 3).to_string());
        assert_eq!("#ffffff", RGB::WHITE.to_string());
        assert_eq!("#ff0000", RGB::RED.to_string());
        assert_eq!("#00ff00", RGB::GREEN.to_string());
        assert_eq!("#0000ff", RGB::BLUE.to_string());
    }

    #[test]
    fn rgba_parse() {
        assert_eq!("#ffffff".parse(), Ok(RGBA::WHITE));
        assert_eq!("#ff0000".parse(), Ok(RGBA::RED));
        assert_eq!("#00ff00".parse(), Ok(RGBA::GREEN));
        assert_eq!("#0000ff".parse(), Ok(RGBA::BLUE));
        assert_eq!("#01020304".parse(), Ok(RGBA::new(1, 2, 3, 4)));
        assert_eq!("#ffffffff".parse(), Ok(RGBA::WHITE));
        assert_eq!("#ff0000ff".parse(), Ok(RGBA::RED));
        assert_eq!("#00ff00ff".parse(), Ok(RGBA::GREEN));
        assert_eq!("#0000ffff".parse(), Ok(RGBA::BLUE));
    }

    #[test]
    fn rgba_format() {
        assert_eq!("#ffffff", RGBA::WHITE.to_string());
        assert_eq!("#ff0000", RGBA::RED.to_string());
        assert_eq!("#00ff00", RGBA::GREEN.to_string());
        assert_eq!("#0000ff", RGBA::BLUE.to_string());
        assert_eq!("#01020304", RGBA::new(1, 2, 3, 4).to_string());
    }

    #[test]
    fn rgb_scale_color() {
        let c = RGB::from_u32_rgb(0xff601f);
        assert_eq!(c.scale_color(0xff), RGB::from_u32_rgb(0xff601f));
        assert_eq!(c.scale_color(0x7f), RGB::from_u32_rgb(0x7f300f));
        assert_eq!(c.scale_color(0x14), RGB::from_u32_rgb(0x140702));
    }

    #[test]
    fn rgba_scale_color() {
        let c = RGBA::from_u32_argb(0xffff601f);
        assert_eq!(c.scale_color(0xff), RGBA::from_u32_argb(0xffff601f));
        assert_eq!(c.scale_color(0x7f), RGBA::from_u32_argb(0xff7f300f));
        assert_eq!(c.scale_color(0x14), RGBA::from_u32_argb(0xff140702));
    }
}
