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
    0x00000000 => BLACK,
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    const fn from_u32_rgb(value: u32) -> Self {
        let [_, r, g, b] = value.to_be_bytes();
        Self::new(r, g, b)
    }

    pub const fn splat(c: u8) -> Self {
        Self::new(c, c, c)
    }

    pub const fn rgba(self, a: u8) -> RGBA {
        RGBA::new(self.r, self.g, self.b, a)
    }

    pub fn saturating_add(self, other: impl Into<RGB>) -> RGB {
        let other = other.into();
        Self::new(
            self.r.saturating_add(other.r),
            self.g.saturating_add(other.g),
            self.b.saturating_add(other.b),
        )
    }

    pub fn scale_color(self, a: u8) -> Self {
        if a == 255 {
            return self;
        }
        let a = a as u16 + 1;
        let r = ((self.r as u16 * a) >> 8) as u8;
        let g = ((self.g as u16 * a) >> 8) as u8;
        let b = ((self.b as u16 * a) >> 8) as u8;
        Self::new(r, g, b)
    }

    #[deprecated(note = "use scale_color instead")]
    pub fn scale(self, a: u8) -> Self {
        self.scale_color(a)
    }

    pub fn blend_alpha(self, other: impl Into<RGB>, alpha: u8) -> RGB {
        let x = self.scale_color(alpha);
        let y = other.into().scale_color(255 - alpha);
        Self::new(x.r + y.r, x.g + y.g, x.b + y.b)
    }

    pub const fn to_rgba(self) -> RGBA {
        RGBA::rgb(self.r, self.g, self.b)
    }

    pub const fn from_bytes([r, g, b]: [u8; 3]) -> RGB {
        Self::new(r, g, b)
    }

    pub const fn to_bytes(self) -> [u8; 3] {
        let Self { r, g, b } = self;
        [r, g, b]
    }
}

impl From<[u8; 3]> for RGB {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::new(r, g, b)
    }
}

impl From<RGBA> for RGB {
    fn from(color: RGBA) -> RGB {
        RGB::new(color.r, color.g, color.b)
    }
}

impl From<RGB> for [u8; 3] {
    fn from(RGB { r, g, b }: RGB) -> Self {
        [r, g, b]
    }
}

impl From<RGB> for [c_int; 3] {
    fn from(RGB { r, g, b }: RGB) -> Self {
        [r as c_int, g as c_int, b as c_int]
    }
}

impl From<color24> for RGB {
    fn from(color: color24) -> RGB {
        RGB::new(color.r, color.g, color.b)
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord, Default)]
#[repr(align(4))]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> RGBA {
        Self { r, g, b, a }
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

    pub const fn to_rgb(self) -> RGB {
        RGB::new(self.r, self.g, self.b)
    }

    pub const fn from_bytes([r, g, b, a]: [u8; 4]) -> RGBA {
        Self::new(r, g, b, a)
    }

    pub const fn to_bytes(self) -> [u8; 4] {
        let Self { r, g, b, a } = self;
        [r, g, b, a]
    }

    pub fn scale_color(self, a: u8) -> RGBA {
        if a == 255 {
            return self;
        }
        let a = a as u16 + 1;
        let r = ((self.r as u16 * a) >> 8) as u8;
        let g = ((self.g as u16 * a) >> 8) as u8;
        let b = ((self.b as u16 * a) >> 8) as u8;
        Self::new(r, g, b, self.a)
    }

    pub fn blend_color(self, color: impl Into<RGB>) -> RGBA {
        let color = color.into();
        let r = ((self.r as u16 * color.r as u16) >> 8) as u8;
        let g = ((self.g as u16 * color.g as u16) >> 8) as u8;
        let b = ((self.b as u16 * color.b as u16) >> 8) as u8;
        Self::new(r, g, b, self.a)
    }

    pub fn blend_color_with_alpha(self, other: impl Into<RGB>, alpha: u8) -> RGBA {
        let x = self.scale_color(alpha);
        let y = other.into().scale_color(255 - alpha);
        Self::new(x.r + y.r, x.g + y.g, x.b + y.b, self.a)
    }

    pub fn blend(self, other: impl Into<RGBA>) -> Self {
        let other = other.into();
        let r = ((self.r as u16 * other.r as u16) >> 8) as u8;
        let g = ((self.g as u16 * other.g as u16) >> 8) as u8;
        let b = ((self.b as u16 * other.b as u16) >> 8) as u8;
        let a = ((self.a as u16 * other.a as u16) >> 8) as u8;
        Self::new(r, g, b, a)
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
        RGBA::rgb(color.r, color.g, color.b)
    }
}

impl From<color24> for RGBA {
    fn from(color: color24) -> RGBA {
        RGBA::rgb(color.r, color.g, color.b)
    }
}

impl From<RGBA> for [u8; 3] {
    fn from(RGBA { r, g, b, .. }: RGBA) -> Self {
        [r, g, b]
    }
}

impl From<RGBA> for [u8; 4] {
    fn from(RGBA { r, g, b, a }: RGBA) -> Self {
        [r, g, b, a]
    }
}

impl From<RGBA> for [c_int; 4] {
    fn from(RGBA { r, g, b, a }: RGBA) -> Self {
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
            ret.a = u8::from_str_radix(&s[7..9], 16)?;
        };
        Ok(ret)
    }
}

impl fmt::Display for RGB {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "#{:02x}{:02x}{:02x}", self.r, self.g, self.b)
    }
}

impl fmt::Display for RGBA {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let [r, g, b, a] = <[u8; 4]>::from(*self);
        if self.a != u8::MAX {
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
