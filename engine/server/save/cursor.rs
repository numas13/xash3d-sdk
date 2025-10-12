use core::{cmp, mem};

#[cfg(feature = "save")]
use crate::save::{Save, SaveState};
use crate::save::{SaveError, SaveResult, Token};

const PREFIX_SIZE_B: u8 = 0xfb;
const PREFIX_SIZE_H: u8 = 0xfc;
const PREFIX_SIZE_W: u8 = 0xfd;
const PREFIX_SIZE_D: u8 = 0xfe;
const PREFIX_SIZE_Q: u8 = 0xff;

// TODO: fill the array with the most frequently used f32 values
const F32_MAP: &[(u8, f32)] = &[
    (0xfe, 0.0),
    (0xfd, 1.0),
    (0xfc, -0.0),
    (0xfb, -1.0),
    (0xfa, 10.0),
    (0xf9, 32.0),
    (0xf8, 34.0),
    // (0xf7, ),
    // (0xf6, ),
    // (0xf5, ),
    // (0xf4, ),
    // (0xf3, ),
    // (0xf2, ),
    // (0xf1, ),
    // (0xf0, ),
    (0x7e, 6.0),
    (0x7d, 50.0),
    (0x7c, 64.0),
    (0x7b, -32.0),
    (0x7a, 3.0),
    (0x79, 18.0),
    (0x78, 150.0),
    // (0x77, ),
    // (0x76, ),
    // (0x75, ),
    // (0x74, ),
    // (0x73, ),
    // (0x72, ),
    // (0x71, ),
    // (0x70, ),
];

// next four bytes is f32_be
const F32_ESCAPE_BYTE: u8 = 0xff;

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

    pub fn cursor(&self) -> Cursor<'a> {
        Cursor::new(self.data())
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

    pub fn read_bool(&mut self) -> SaveResult<bool> {
        self.read_u8().map(|i| i != 0)
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

    pub fn read_leb_u8(&mut self) -> SaveResult<u8> {
        self.read_leb_u128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_u16(&mut self) -> SaveResult<u16> {
        self.read_leb_u128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_u32(&mut self) -> SaveResult<u32> {
        self.read_leb_u128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_u64(&mut self) -> SaveResult<u64> {
        self.read_leb_u128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_u128(&mut self) -> SaveResult<u128> {
        match self.read_u8()? {
            PREFIX_SIZE_Q => self.read_u128_le(),
            PREFIX_SIZE_D => self.read_u64_le().map(|value| value.into()),
            PREFIX_SIZE_W => self.read_u32_le().map(|value| value.into()),
            PREFIX_SIZE_H => self.read_u16_le().map(|value| value.into()),
            PREFIX_SIZE_B => self.read_u8().map(|value| value.into()),
            value => Ok(value.into()),
        }
    }

    #[cfg(target_pointer_width = "16")]
    pub fn read_leb_usize(&mut self) -> SaveResult<usize> {
        self.read_leb_u16().map(|value| value as usize)
    }

    #[cfg(target_pointer_width = "32")]
    pub fn read_leb_usize(&mut self) -> SaveResult<usize> {
        self.read_leb_u32().map(|value| value as usize)
    }

    #[cfg(target_pointer_width = "64")]
    pub fn read_leb_usize(&mut self) -> SaveResult<usize> {
        self.read_leb_u64().map(|value| value as usize)
    }

    pub fn read_leb_i8(&mut self) -> SaveResult<i8> {
        self.read_leb_i128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_i16(&mut self) -> SaveResult<i16> {
        self.read_leb_i128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_i32(&mut self) -> SaveResult<i32> {
        self.read_leb_i128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_i64(&mut self) -> SaveResult<i64> {
        self.read_leb_i128()
            .and_then(|value| value.try_into().map_err(|_| SaveError::InvalidNumber))
    }

    pub fn read_leb_i128(&mut self) -> SaveResult<i128> {
        match self.read_u8()? {
            PREFIX_SIZE_Q => self.read_i128_le(),
            PREFIX_SIZE_D => self.read_i64_le().map(|value| value.into()),
            PREFIX_SIZE_W => self.read_i32_le().map(|value| value.into()),
            PREFIX_SIZE_H => self.read_i16_le().map(|value| value.into()),
            PREFIX_SIZE_B => self.read_i8().map(|value| value.into()),
            value => Ok(value.into()),
        }
    }

    #[cfg(target_pointer_width = "16")]
    pub fn read_leb_isize(&mut self) -> SaveResult<isize> {
        self.read_leb_i16().map(|value| value as isize)
    }

    #[cfg(target_pointer_width = "32")]
    pub fn read_leb_isize(&mut self) -> SaveResult<isize> {
        self.read_leb_i32().map(|value| value as isize)
    }

    #[cfg(target_pointer_width = "64")]
    pub fn read_leb_isize(&mut self) -> SaveResult<isize> {
        self.read_leb_i64().map(|value| value as isize)
    }

    pub fn read_f32(&mut self) -> SaveResult<f32> {
        let b0 = self.read_u8()?;
        if b0 == F32_ESCAPE_BYTE {
            return self.read_f32_be();
        }

        for &(byte, fixed) in F32_MAP {
            if b0 == byte {
                return Ok(fixed);
            }
        }

        let [b1, b2, b3] = self.read_array()?;
        Ok(f32::from_be_bytes([b0, b1, b2, b3]))
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

    pub fn read_bytes_with_size(&mut self) -> SaveResult<&'a [u8]> {
        let len = self.read_leb_usize()?;
        self.read(len)
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

    pub fn write_bool(&mut self, value: bool) -> SaveResult<usize> {
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

    pub fn write_leb_u8(&mut self, value: u8) -> SaveResult<()> {
        if value >= PREFIX_SIZE_B {
            self.write_u8(PREFIX_SIZE_B)?;
        }
        self.write_u8(value)?;
        Ok(())
    }

    pub fn write_leb_u16(&mut self, value: u16) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_u8(value);
        }
        self.write_u8(PREFIX_SIZE_H)?;
        self.write_u16_le(value)?;
        Ok(())
    }

    pub fn write_leb_u32(&mut self, value: u32) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_u16(value);
        }
        self.write_u8(PREFIX_SIZE_W)?;
        self.write_u32_le(value)?;
        Ok(())
    }

    pub fn write_leb_u64(&mut self, value: u64) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_u32(value);
        }
        self.write_u8(PREFIX_SIZE_D)?;
        self.write_u64_le(value)?;
        Ok(())
    }

    pub fn write_leb_u128(&mut self, value: u128) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_u64(value);
        }
        self.write_u8(PREFIX_SIZE_Q)?;
        self.write_u128_le(value)?;
        Ok(())
    }

    #[cfg(target_pointer_width = "16")]
    pub fn write_leb_usize(&mut self, value: usize) -> SaveResult<()> {
        self.write_leb_u16(value as u16)
    }

    #[cfg(target_pointer_width = "32")]
    pub fn write_leb_usize(&mut self, value: usize) -> SaveResult<()> {
        self.write_leb_u32(value as u32)
    }

    #[cfg(target_pointer_width = "64")]
    pub fn write_leb_usize(&mut self, value: usize) -> SaveResult<()> {
        self.write_leb_u64(value as u64)
    }

    pub fn write_leb_i8(&mut self, value: i8) -> SaveResult<()> {
        if value <= (PREFIX_SIZE_B - 127) as i8 {
            self.write_u8(PREFIX_SIZE_B)?;
        }
        self.write_i8(value)?;
        Ok(())
    }

    pub fn write_leb_i16(&mut self, value: i16) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_i8(value);
        }
        self.write_u8(PREFIX_SIZE_H)?;
        self.write_i16_le(value)?;
        Ok(())
    }

    pub fn write_leb_i32(&mut self, value: i32) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_i16(value);
        }
        self.write_u8(PREFIX_SIZE_W)?;
        self.write_i32_le(value)?;
        Ok(())
    }

    pub fn write_leb_i64(&mut self, value: i64) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_i32(value);
        }
        self.write_u8(PREFIX_SIZE_D)?;
        self.write_i64_le(value)?;
        Ok(())
    }

    pub fn write_leb_i128(&mut self, value: i128) -> SaveResult<()> {
        if let Ok(value) = value.try_into() {
            return self.write_leb_i64(value);
        }
        self.write_u8(PREFIX_SIZE_Q)?;
        self.write_i128_le(value)?;
        Ok(())
    }

    #[cfg(target_pointer_width = "16")]
    pub fn write_leb_isize(&mut self, value: isize) -> SaveResult<()> {
        self.write_leb_i16(value as i16)
    }

    #[cfg(target_pointer_width = "32")]
    pub fn write_leb_isize(&mut self, value: isize) -> SaveResult<()> {
        self.write_leb_i32(value as i32)
    }

    #[cfg(target_pointer_width = "64")]
    pub fn write_leb_isize(&mut self, value: isize) -> SaveResult<()> {
        self.write_leb_i64(value as i64)
    }

    pub fn write_f32(&mut self, value: f32) -> SaveResult<()> {
        let bytes = value.to_be_bytes();
        let mut matched = false;
        for &(byte, fixed) in F32_MAP {
            if byte == bytes[0] {
                matched = true;
            }
            if value == fixed {
                self.write_u8(byte)?;
                return Ok(());
            }
        }
        if matched {
            self.write_u8(F32_ESCAPE_BYTE)?;
        }
        self.write(&bytes)?;
        Ok(())
    }

    pub fn write_token(&mut self, token: Token) -> SaveResult<usize> {
        self.write_u16_le(token.to_u16())
    }

    #[cfg(feature = "save")]
    pub fn write_field<T: Save + ?Sized>(
        &mut self,
        state: &mut SaveState,
        name: &'static core::ffi::CStr,
        value: &T,
    ) -> SaveResult<()> {
        let header_offset = self.skip(4)?;
        value.save(state, self)?;
        let size = self.offset() - header_offset - 4;
        let size = size.try_into().map_err(|_| SaveError::SizeOverflow)?;
        self.write_at(header_offset, |cur| {
            cur.write_u16_le(size)?;
            cur.write_token(state.token_hash(name))?;
            Ok(())
        })
    }

    pub fn write_bytes_with_size(&mut self, bytes: &[u8]) -> SaveResult<usize> {
        let offset = self.offset();
        self.write_leb_usize(bytes.len())?;
        self.write(bytes)?;
        Ok(self.offset() - offset)
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

    #[test]
    fn crusor_leb_u8() {
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in 0..=255 {
            cur.write_leb_u8(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in 0..=255 {
            assert_eq!(cur.read_leb_u8(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_u16() {
        let numbers = [0, 250, u8::MAX as u16, u8::MAX as u16 + 1, u16::MAX];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_u16(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_u16(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_u32() {
        let numbers = [
            0,
            250,
            u8::MAX as u32,
            u8::MAX as u32 + 1,
            u16::MAX as u32,
            u16::MAX as u32 + 1,
            u32::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_u32(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_u32(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_u64() {
        let numbers = [
            0,
            250,
            u8::MAX as u64,
            u8::MAX as u64 + 1,
            u16::MAX as u64,
            u16::MAX as u64 + 1,
            u32::MAX as u64,
            u32::MAX as u64 + 1,
            u64::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_u64(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_u64(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_u128() {
        let numbers = [
            0,
            250,
            u8::MAX as u128,
            u8::MAX as u128 + 1,
            u16::MAX as u128,
            u16::MAX as u128 + 1,
            u32::MAX as u128,
            u32::MAX as u128 + 1,
            u64::MAX as u128,
            u64::MAX as u128 + 1,
            u128::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_u128(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_u128(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_i8() {
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in 0..=255 {
            cur.write_leb_i8(i as i8).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in 0..=255 {
            assert_eq!(cur.read_leb_i8(), Ok(i as i8));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_i16() {
        let numbers = [
            0,
            122,
            i8::MIN as i16,
            i8::MAX as i16,
            i8::MIN as i16 - 1,
            i8::MAX as i16 + 1,
            i16::MIN,
            i16::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_i16(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_i16(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_i32() {
        let numbers = [
            0,
            122,
            i8::MIN as i32,
            i8::MAX as i32,
            i8::MIN as i32 - 1,
            i8::MAX as i32 + 1,
            i16::MIN as i32,
            i16::MAX as i32,
            i16::MIN as i32 - 1,
            i16::MAX as i32 + 1,
            i32::MIN,
            i32::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_i32(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_i32(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_i64() {
        let numbers = [
            0,
            122,
            i8::MIN as i64,
            i8::MAX as i64,
            i8::MIN as i64 - 1,
            i8::MAX as i64 + 1,
            i16::MIN as i64,
            i16::MAX as i64,
            i16::MIN as i64 - 1,
            i16::MAX as i64 + 1,
            i32::MIN as i64,
            i32::MAX as i64,
            i32::MIN as i64 - 1,
            i32::MAX as i64 + 1,
            i64::MIN,
            i64::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_i64(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_i64(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn crusor_leb_i128() {
        let numbers = [
            0,
            122,
            i8::MIN as i128,
            i8::MAX as i128,
            i8::MIN as i128 - 1,
            i8::MAX as i128 + 1,
            i16::MIN as i128,
            i16::MAX as i128,
            i16::MIN as i128 - 1,
            i16::MAX as i128 + 1,
            i32::MIN as i128,
            i32::MAX as i128,
            i32::MIN as i128 - 1,
            i32::MAX as i128 + 1,
            i64::MIN as i128,
            i64::MAX as i128,
            i64::MIN as i128 - 1,
            i64::MAX as i128 + 1,
            i128::MIN,
            i128::MAX,
        ];
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        for i in numbers {
            cur.write_leb_i128(i).unwrap();
        }
        let mut cur = Cursor::new(&buf);
        for i in numbers {
            assert_eq!(cur.read_leb_i128(), Ok(i));
        }
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }

    #[test]
    fn cursor_leb_f32() {
        let mut buf = [0xaa; 1024];
        let mut cur = CursorMut::new(&mut buf);
        cur.write_f32(-1.123456).unwrap();
        for &(_, fixed) in F32_MAP {
            cur.write_f32(fixed).unwrap();
        }
        cur.write_f32(2.34567).unwrap();
        cur.write_f32(f32::NAN).unwrap();

        let mut cur = Cursor::new(&buf);
        assert_eq!(cur.read_f32(), Ok(-1.123456));
        for &(_, fixed) in F32_MAP {
            assert_eq!(cur.read_f32(), Ok(fixed));
        }
        assert_eq!(cur.read_f32(), Ok(2.34567));
        assert_eq!(cur.read_f32().map(|f| f.is_nan()), Ok(true));
        assert_eq!(cur.read_u8(), Ok(0xaa));
    }
}
