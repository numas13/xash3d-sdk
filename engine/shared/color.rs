use core::{ffi::c_int, fmt, str::FromStr};

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct RGB {
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

impl RGB {
    pub const BLACK: Self = Self::from_u32(0x000000);
    pub const SILVER: Self = Self::from_u32(0xc0c0c0);
    pub const GRAY: Self = Self::from_u32(0x808080);
    pub const WHITE: Self = Self::from_u32(0xffffff);
    pub const MAROON: Self = Self::from_u32(0x800000);
    pub const RED: Self = Self::from_u32(0xff0000);
    pub const GREEN: Self = Self::from_u32(0x00ff00);
    pub const LIME: Self = Self::from_u32(0x00ff00);
    pub const NAVY: Self = Self::from_u32(0x000080);
    pub const BLUE: Self = Self::from_u32(0x0000ff);
    pub const YELLOWISH: Self = Self::from_u32(0xffa000);
    pub const REDISH: Self = Self::from_u32(0xff1010);
    pub const GREENISH: Self = Self::from_u32(0x00a000);
    pub const PURPLE: Self = Self::from_u32(0x800080);
    pub const FUCHSIA: Self = Self::from_u32(0xff00ff);
    pub const CYAN: Self = Self::from_u32(0x00ffff);

    pub const fn new(r: u8, g: u8, b: u8) -> Self {
        Self { r, g, b }
    }

    pub const fn from_u32(value: u32) -> Self {
        let r = (value >> 16) as u8;
        let g = (value >> 8) as u8;
        let b = value as u8;
        Self::new(r, g, b)
    }

    pub const fn splat(c: u8) -> Self {
        Self::new(c, c, c)
    }

    pub const fn rgba(self, a: u8) -> RGBA {
        RGBA::new(self.r, self.g, self.b, a)
    }

    pub fn scale(&self, a: u8) -> Self {
        let a = a as f32 / 255.0;
        let r = (self.r as f32 * a) as u8;
        let g = (self.g as f32 * a) as u8;
        let b = (self.b as f32 * a) as u8;
        Self::new(r, g, b)
    }

    pub const fn to_rgba(&self) -> RGBA {
        RGBA::rgb(self.r, self.g, self.b)
    }
}

impl From<[u8; 3]> for RGB {
    fn from([r, g, b]: [u8; 3]) -> Self {
        Self::new(r, g, b)
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub struct RGBA {
    pub r: u8,
    pub g: u8,
    pub b: u8,
    pub a: u8,
}

impl RGBA {
    pub const BLACK: Self = Self::splat(0);
    pub const WHITE: Self = Self::splat(255);
    pub const RED: Self = Self::rgb(255, 0, 0);
    pub const GREEN: Self = Self::rgb(0, 255, 0);
    pub const BLUE: Self = Self::rgb(0, 0, 255);

    pub const fn new(r: u8, g: u8, b: u8, a: u8) -> Self {
        Self { r, g, b, a }
    }

    pub const fn rgb(r: u8, g: u8, b: u8) -> Self {
        Self::new(r, g, b, 255)
    }

    pub const fn splat(c: u8) -> Self {
        Self::rgb(c, c, c)
    }

    pub const fn to_rgb(&self) -> RGB {
        RGB::new(self.r, self.g, self.b)
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
}
