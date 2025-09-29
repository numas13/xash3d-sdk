use core::{cmp, mem};

use crate::save::{SaveError, SaveResult, Token};

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq)]
pub struct Header {
    size: u16,
    token: Token,
}

impl Header {
    pub fn new(size: u16, token: Token) -> Self {
        Self { size, token }
    }

    pub fn token(&self) -> Token {
        self.token
    }

    pub fn size(&self) -> u16 {
        self.size
    }
}

#[derive(Copy, Clone, Default, Debug)]
pub struct Field<'a> {
    token: Token,
    data: &'a [u8],
}

impl<'a> Field<'a> {
    pub fn new(token: Token, data: &'a [u8]) -> SaveResult<Self> {
        if u16::try_from(data.len()).is_ok() {
            Ok(Self { token, data })
        } else {
            Err(SaveError::Overflow)
        }
    }

    pub fn token(&self) -> Token {
        self.token
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn size(&self) -> usize {
        self.data.len()
    }

    pub fn header(&self) -> Header {
        Header::new(self.data.len() as u16, self.token)
    }
}

macro_rules! impl_read_num {
    ($( $from_bytes:ident { $( fn $name:ident() -> $ty:ty ),* $(,)? } )*) => {
        $(
            $(
                pub fn $name(&mut self) -> SaveResult<$ty> {
                    self.read_array().map(<$ty>::$from_bytes)
                }
            )*
        )*
    };
}

macro_rules! impl_write_num {
    ($( $to_bytes:ident { $( fn $name:ident($ty:ty) ),* $(,)? } )*) => {
        $(
            $(
                pub fn $name(&mut self, value: $ty) -> SaveResult<usize> {
                    self.write_array(value.$to_bytes())
                }
            )*
        )*
    };
}

#[derive(Copy, Clone, Default)]
pub struct Cursor<'a> {
    buffer: &'a [u8],
    offset: usize,
}

impl<'a> Cursor<'a> {
    pub fn new(buffer: &'a [u8]) -> Self {
        Self { buffer, offset: 0 }
    }

    pub fn with_offset(buffer: &'a [u8], offset: usize) -> SaveResult<Self> {
        if offset <= buffer.len() {
            Ok(Self { buffer, offset })
        } else {
            Err(SaveError::Overflow)
        }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn set_offset(&mut self, offset: usize) -> SaveResult<usize> {
        if offset <= self.buffer.len() {
            Ok(mem::replace(&mut self.offset, offset))
        } else {
            Err(SaveError::Overflow)
        }
    }

    #[inline]
    pub fn remaining(&self) -> usize {
        self.buffer.len() - self.offset
    }

    #[inline]
    pub fn is_empty(&self) -> bool {
        self.remaining() == 0
    }

    pub fn as_slice(&self) -> &'a [u8] {
        &self.buffer[self.offset..]
    }

    pub fn check_remaining(&self, len: usize) -> SaveResult<usize> {
        let new_offset = self.offset + len;
        if self.buffer.len() < new_offset {
            Err(SaveError::Overflow)
        } else {
            Ok(new_offset)
        }
    }

    pub fn read(&mut self, len: usize) -> SaveResult<&'a [u8]> {
        let new_offset = self.check_remaining(len)?;
        let bytes = &self.buffer[self.offset..new_offset];
        self.offset = new_offset;
        Ok(bytes)
    }

    #[inline]
    pub fn read_array<const N: usize>(&mut self) -> SaveResult<[u8; N]> {
        let mut tmp = [0; N];
        tmp.copy_from_slice(self.read(N)?);
        Ok(tmp)
    }

    pub fn read_u8(&mut self) -> SaveResult<u8> {
        self.read(1).map(|i| i[0])
    }

    pub fn read_i8(&mut self) -> SaveResult<i8> {
        self.read_u8().map(|i| i as i8)
    }

    impl_read_num! {
        from_ne_bytes {
            fn read_u16_ne() -> u16,
            fn read_u32_ne() -> u32,
            fn read_u64_ne() -> u64,
            fn read_u128_ne() -> u128,
            fn read_usize_ne() -> usize,

            fn read_i16_ne() -> i16,
            fn read_i32_ne() -> i32,
            fn read_i64_ne() -> i64,
            fn read_i128_ne() -> i128,
            fn read_isize_ne() -> isize,

            fn read_f32_ne() -> f32,
            fn read_f64_ne() -> f64,
        }
        from_le_bytes {
            fn read_u16_le() -> u16,
            fn read_u32_le() -> u32,
            fn read_u64_le() -> u64,
            fn read_u128_le() -> u128,
            fn read_usize_le() -> usize,

            fn read_i16_le() -> i16,
            fn read_i32_le() -> i32,
            fn read_i64_le() -> i64,
            fn read_i128_le() -> i128,
            fn read_isize_le() -> isize,

            fn read_f32_le() -> f32,
            fn read_f64_le() -> f64,
        }
        from_be_bytes {
            fn read_u16_be() -> u16,
            fn read_u32_be() -> u32,
            fn read_u64_be() -> u64,
            fn read_u128_be() -> u128,
            fn read_usize_be() -> usize,

            fn read_i16_be() -> i16,
            fn read_i32_be() -> i32,
            fn read_i64_be() -> i64,
            fn read_i128_be() -> i128,
            fn read_isize_be() -> isize,

            fn read_f32_be() -> f32,
            fn read_f64_be() -> f64,
        }
    }

    pub fn read_header(&mut self) -> SaveResult<Header> {
        let size = self.read_u16_le()?;
        let token = Token::new(self.read_u16_le()?);
        Ok(Header::new(size, token))
    }

    pub fn read_field(&mut self) -> SaveResult<Field<'a>> {
        let header = self.read_header()?;
        let data = self.read(header.size() as usize)?;
        Field::new(header.token(), data)
    }
}

pub struct CursorMut<'a> {
    buffer: &'a mut [u8],
    offset: usize,
}

impl<'a> CursorMut<'a> {
    pub fn new(buffer: &'a mut [u8]) -> Self {
        Self { buffer, offset: 0 }
    }

    pub fn with_offset(buffer: &'a mut [u8], offset: usize) -> SaveResult<Self> {
        if offset <= buffer.len() {
            Ok(Self { buffer, offset })
        } else {
            Err(SaveError::Overflow)
        }
    }

    #[inline]
    pub fn offset(&self) -> usize {
        self.offset
    }

    #[inline]
    pub fn set_offset(&mut self, offset: usize) -> SaveResult<usize> {
        if offset <= self.buffer.len() {
            Ok(mem::replace(&mut self.offset, offset))
        } else {
            Err(SaveError::Overflow)
        }
    }

    #[inline]
    pub fn skip(&mut self, len: usize) -> SaveResult<usize> {
        self.set_offset(self.offset() + len)
    }

    #[inline]
    pub fn capacity(&self) -> usize {
        self.buffer.len()
    }

    #[inline]
    pub fn avaiable(&self) -> usize {
        self.buffer.len() - self.offset
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.buffer[..self.offset()]
    }

    pub fn as_slice_mut(&mut self) -> &mut [u8] {
        let size = self.offset();
        &mut self.buffer[..size]
    }

    pub fn check_available(&self, size: usize) -> SaveResult<usize> {
        let new_offset = self.offset + size;
        if self.buffer.len() < new_offset {
            return Err(SaveError::Overflow);
        }
        Ok(new_offset)
    }

    pub fn write_at<T>(
        &mut self,
        offset: usize,
        f: impl FnOnce(&mut Self) -> SaveResult<T>,
    ) -> SaveResult<T> {
        if self.offset < offset {
            return Err(SaveError::Overflow);
        }
        let old_offset = self.set_offset(offset)?;
        match f(self) {
            Ok(value) => {
                self.offset = cmp::max(self.offset, old_offset);
                Ok(value)
            }
            Err(err) => {
                self.offset = old_offset;
                Err(err)
            }
        }
    }

    pub fn write(&mut self, bytes: &[u8]) -> SaveResult<usize> {
        let new_offset = self.check_available(bytes.len())?;
        self.buffer[self.offset..new_offset].copy_from_slice(bytes);
        self.offset = new_offset;
        Ok(bytes.len())
    }

    #[inline]
    pub fn write_array<const N: usize>(&mut self, array: [u8; N]) -> SaveResult<usize> {
        self.write(&array)
    }

    pub fn write_u8(&mut self, value: u8) -> SaveResult<usize> {
        self.write(&[value])
    }

    pub fn write_i8(&mut self, value: i8) -> SaveResult<usize> {
        self.write_u8(value as u8)
    }

    impl_write_num! {
        to_ne_bytes {
            fn write_u16_ne(u16),
            fn write_u32_ne(u32),
            fn write_u64_ne(u64),
            fn write_u128_ne(u128),
            fn write_usize_ne(usize),

            fn write_i16_ne(i16),
            fn write_i32_ne(i32),
            fn write_i64_ne(i64),
            fn write_i128_ne(i128),
            fn write_isize_ne(isize),

            fn write_f32_ne(f32),
            fn write_f64_ne(f64),
        }
        to_le_bytes {
            fn write_u16_le(u16),
            fn write_u32_le(u32),
            fn write_u64_le(u64),
            fn write_u128_le(u128),
            fn write_usize_le(usize),

            fn write_i16_le(i16),
            fn write_i32_le(i32),
            fn write_i64_le(i64),
            fn write_i128_le(i128),
            fn write_isize_le(isize),

            fn write_f32_le(f32),
            fn write_f64_le(f64),
        }
        to_be_bytes {
            fn write_u16_be(u16),
            fn write_u32_be(u32),
            fn write_u64_be(u64),
            fn write_u128_be(u128),
            fn write_usize_be(usize),

            fn write_i16_be(i16),
            fn write_i32_be(i32),
            fn write_i64_be(i64),
            fn write_i128_be(i128),
            fn write_isize_be(isize),

            fn write_f32_be(f32),
            fn write_f64_be(f64),
        }
    }

    pub fn write_token(&mut self, token: Token) -> SaveResult<usize> {
        self.write_u16_le(token.to_u16())
    }

    pub fn write_header(&mut self, header: Header) -> SaveResult<()> {
        self.write_u16_le(header.size)?;
        self.write_token(header.token())?;
        Ok(())
    }

    pub fn write_field(&mut self, field: Field) -> SaveResult<()> {
        self.write_header(field.header())?;
        self.write(field.data())?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cursor_read() {
        let src = b"foobar 123456789";
        let mut cur = Cursor::new(src);
        assert_eq!(cur.read(3), Ok(&b"foo"[..]));
        assert_eq!(cur.read(4), Ok(&b"bar "[..]));
        assert_eq!(cur.read(7), Ok(&b"1234567"[..]));
        assert_eq!(cur.read_u8(), Ok(b'8'));
        assert_eq!(cur.read_i8(), Ok(b'9' as i8));
        assert_eq!(cur.remaining(), 0);
    }

    macro_rules! test_read_num {
        ($(
            $value:expr, $to:ident {
                $( $read:ident() -> $ty:ty ),* $(,)?
            }
        )*) => {
            $(
                $(
                    let n = $value as $ty;
                    let src = n.$to();
                    assert_eq!(Cursor::new(&src).$read(), Ok(n));
                )*
            )*
        }
    }

    #[test]
    fn cursor_read_int() {
        test_read_num! {
            0x12345678deadbeef_u128, to_ne_bytes {
                read_u16_ne() -> u16,
                read_u32_ne() -> u32,
                read_u64_ne() -> u64,
                read_u128_ne() -> u128,
                read_usize_ne() -> usize,

                read_i16_ne() -> i16,
                read_i32_ne() -> i32,
                read_i64_ne() -> i64,
                read_i128_ne() -> i128,
                read_isize_ne() -> isize,
            }
            0x12345678deadbeef_u128, to_le_bytes {
                read_u16_le() -> u16,
                read_u32_le() -> u32,
                read_u64_le() -> u64,
                read_u128_le() -> u128,
                read_usize_le() -> usize,

                read_i16_le() -> i16,
                read_i32_le() -> i32,
                read_i64_le() -> i64,
                read_i128_le() -> i128,
                read_isize_le() -> isize,
            }
            0x12345678deadbeef_u128, to_be_bytes {
                read_u16_be() -> u16,
                read_u32_be() -> u32,
                read_u64_be() -> u64,
                read_u128_be() -> u128,
                read_usize_be() -> usize,

                read_i16_be() -> i16,
                read_i32_be() -> i32,
                read_i64_be() -> i64,
                read_i128_be() -> i128,
                read_isize_be() -> isize,
            }
        }
    }

    #[test]
    fn cursor_read_fp() {
        test_read_num! {
            core::f64::consts::PI, to_ne_bytes {
                read_f32_ne() -> f32,
                read_f64_ne() -> f64,
            }
            core::f64::consts::PI, to_le_bytes {
                read_f32_le() -> f32,
                read_f64_le() -> f64,
            }
            core::f64::consts::PI, to_be_bytes {
                read_f32_be() -> f32,
                read_f64_be() -> f64,
            }
        }
    }

    #[test]
    fn cursor_write() {
        let mut dst = [0; 16];
        let mut cur = CursorMut::new(&mut dst);
        assert_eq!(cur.write(b"foo"), Ok(3));
        assert_eq!(cur.write(b"bar "), Ok(4));
        assert_eq!(cur.write(b"1234567"), Ok(7));
        assert_eq!(cur.write_u8(b'8'), Ok(1));
        assert_eq!(cur.write_i8(b'9' as i8), Ok(1));
        assert_eq!(cur.offset(), 16);
    }

    macro_rules! test_write_num {
        ($(
            $value:expr, $to:ident {
                $( $write:ident($ty:ty) ),* $(,)?
            }
        )*) => {
            let mut dst = [0; 32];
            let mut cur = CursorMut::new(&mut dst);
            $(
                $(
                    let n = $value as $ty;
                    assert!(cur.set_offset(0).is_ok());
                    assert_eq!(cur.$write(n), Ok(core::mem::size_of::<$ty>()));
                    assert_eq!(cur.as_slice(), n.$to().as_slice());
                )*
            )*
        }
    }

    #[test]
    fn cursor_write_int() {
        test_write_num! {
            0x12345678deadbeef_u128, to_ne_bytes {
                write_u16_ne(u16),
                write_u32_ne(u32),
                write_u64_ne(u64),
                write_u128_ne(u128),
                write_usize_ne(usize),

                write_i16_ne(i16),
                write_i32_ne(i32),
                write_i64_ne(i64),
                write_i128_ne(i128),
                write_isize_ne(isize),
            }
            0x12345678deadbeef_u128, to_le_bytes {
                write_u16_le(u16),
                write_u32_le(u32),
                write_u64_le(u64),
                write_u128_le(u128),
                write_usize_le(usize),

                write_i16_le(i16),
                write_i32_le(i32),
                write_i64_le(i64),
                write_i128_le(i128),
                write_isize_le(isize),
            }
            0x12345678deadbeef_u128, to_be_bytes {
                write_u16_be(u16),
                write_u32_be(u32),
                write_u64_be(u64),
                write_u128_be(u128),
                write_usize_be(usize),

                write_i16_be(i16),
                write_i32_be(i32),
                write_i64_be(i64),
                write_i128_be(i128),
                write_isize_be(isize),
            }
        }
    }

    #[test]
    fn cursor_write_fp() {
        test_write_num! {
            core::f64::consts::PI, to_ne_bytes {
                write_f32_ne(f32),
                write_f64_ne(f64),
            }
            core::f64::consts::PI, to_le_bytes {
                write_f32_le(f32),
                write_f64_le(f64),
            }
            core::f64::consts::PI, to_be_bytes {
                write_f32_be(f32),
                write_f64_be(f64),
            }
        }
    }

    #[test]
    fn cursor_write_at() {
        let mut dst = [b'x'; 8];
        let mut cur = CursorMut::new(&mut dst);
        cur.write(b"foobar").unwrap();

        assert_eq!(cur.as_slice(), b"foobar");
        cur.write_at(0, |cur| cur.write(b"BAR")).unwrap();
        assert_eq!(cur.as_slice(), b"BARbar");
        cur.write_at(3, |cur| cur.write(b"FOO")).unwrap();
        assert_eq!(cur.as_slice(), b"BARFOO");

        // an error is returned if we try to set the offset higher than the current offset
        assert_eq!(cur.write_at(7, |_| Ok(())), Err(SaveError::Overflow));

        // always restore the previous offset on errors
        let old_offset = cur.offset();
        let res = cur.write_at(5, |cur| {
            for _ in 0..cur.avaiable() + 1 {
                cur.write_u8(b'0')?;
            }
            Ok(())
        });
        assert_eq!(res, Err(SaveError::Overflow));
        assert_eq!(cur.offset(), old_offset);

        // this will increace the size
        cur.write_at(5, |cur| cur.write(b"OY")).unwrap();
        assert_eq!(cur.as_slice(), b"BARFOOY");

        cur.write(b"Z").unwrap();
        assert_eq!(cur.as_slice(), b"BARFOOYZ");
    }
}
