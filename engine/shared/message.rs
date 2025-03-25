use core::{
    cmp,
    ffi::{c_int, CStr},
    fmt::{self, Write},
    str,
};

pub type Result<T, E = MessageError> = core::result::Result<T, E>;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct MessageError;

pub trait MessageResult {
    fn convert(self) -> c_int;
}

impl MessageResult for bool {
    fn convert(self) -> c_int {
        self as c_int
    }
}

impl MessageResult for c_int {
    fn convert(self) -> c_int {
        self
    }
}

impl MessageResult for Option<()> {
    fn convert(self) -> c_int {
        self.is_some() as c_int
    }
}

impl MessageResult for Result<(), MessageError> {
    fn convert(self) -> c_int {
        self.is_ok() as c_int
    }
}

impl MessageResult for Result<bool, MessageError> {
    fn convert(self) -> c_int {
        match self {
            Ok(value) => value as c_int,
            Err(_) => 0,
        }
    }
}

#[derive(Copy, Clone)]
pub struct Message<'a> {
    name: &'a CStr,
    data: &'a [u8],
    offset: usize,
}

#[allow(dead_code)]
impl<'a> Message<'a> {
    pub fn new(name: &'a CStr, data: &'a [u8]) -> Self {
        Self {
            name,
            data,
            offset: 0,
        }
    }

    pub fn name(&self) -> &'a CStr {
        self.name
    }

    pub fn data(&self) -> &'a [u8] {
        self.data
    }

    pub fn as_slice(&self) -> &'a [u8] {
        &self.data[self.offset..]
    }

    pub fn skip(&mut self, offset: usize) {
        self.offset = cmp::min(offset, self.data.len());
    }

    pub fn read_u8(&mut self) -> Result<u8> {
        self.data
            .get(self.offset)
            .map(|&byte| {
                self.offset += 1;
                byte
            })
            .ok_or(MessageError)
    }

    pub fn read_i8(&mut self) -> Result<i8> {
        self.read_u8().map(|i| i as i8)
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8]> {
        if self.offset + len <= self.data.len() {
            let ret = &self.data[self.offset..self.offset + len];
            self.offset += len;
            Ok(ret)
        } else {
            Err(MessageError)
        }
    }

    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N]> {
        self.read_slice(N).map(|s| {
            let mut buf = [0; N];
            buf.copy_from_slice(s);
            buf
        })
    }

    pub fn read_u16(&mut self) -> Result<u16> {
        self.read_array().map(u16::from_le_bytes)
    }

    pub fn read_i16(&mut self) -> Result<i16> {
        self.read_array().map(i16::from_le_bytes)
    }

    pub fn read_u32(&mut self) -> Result<u32> {
        self.read_array().map(u32::from_le_bytes)
    }

    pub fn read_i32(&mut self) -> Result<i32> {
        self.read_array().map(i32::from_le_bytes)
    }

    pub fn read_f32(&mut self) -> Result<f32> {
        self.read_array().map(f32::from_le_bytes)
    }

    pub fn read_coord(&mut self) -> Result<f32> {
        self.read_i16().map(|i| i as f32 * (1.0 / 8.0))
    }

    pub fn read_angle(&mut self) -> Result<f32> {
        self.read_i8().map(|i| i as f32 * (360.0 / 256.0))
    }

    pub fn read_hires_angle(&mut self) -> Result<f32> {
        self.read_i16().map(|i| i as f32 * (360.0 / 65536.0))
    }

    pub fn read_cstr(&mut self) -> Result<&'a CStr> {
        let s = CStr::from_bytes_until_nul(self.as_slice()).map_err(|_| MessageError)?;
        self.offset += s.to_bytes_with_nul().len();
        Ok(s)
    }

    pub fn read_str(&mut self) -> Result<&'a str> {
        let tail = self.as_slice();
        let len = tail.iter().position(|&i| i == b'\0').unwrap_or(tail.len());
        let end = self.offset + len;

        let s = str::from_utf8(&self.data[self.offset..end]).map_err(|_| MessageError)?;
        self.offset = end + 1;
        Ok(s)
    }
}

impl fmt::Display for Message<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.write_char('"')?;
        for &i in self.data {
            if i.is_ascii_graphic() {
                fmt.write_char(i as char)?;
            } else {
                write!(fmt, "\\x{i:02}")?;
            }
        }
        fmt.write_char('"')?;
        Ok(())
    }
}

impl fmt::Debug for Message<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.data, fmt)
    }
}

#[derive(Copy, Clone)]
pub struct MessageBuilder<const N: usize = 1024> {
    data: [u8; N],
    offset: usize,
}

macro_rules! impl_write_num {
    ($($name:ident: $ty:ty),+ $(,)?) => {
        $(pub fn $name(&mut self, value: $ty) {
            self.write_array(value.to_le_bytes())
        })+
    };
}

impl<const N: usize> MessageBuilder<N> {
    pub fn new() -> Self {
        Self {
            data: [0; N],
            offset: 0,
        }
    }

    pub fn as_slice(&self) -> &[u8] {
        &self.data[..self.offset]
    }

    pub fn write(&mut self, s: &[u8]) {
        self.data[self.offset..self.offset + s.len()].copy_from_slice(s);
        self.offset += s.len();
    }

    pub fn write_array<const I: usize>(&mut self, array: [u8; I]) {
        self.write(&array)
    }

    pub fn write_u8(&mut self, value: u8) {
        self.write_array([value])
    }

    pub fn write_i8(&mut self, value: i8) {
        self.write_u8(value as u8)
    }

    impl_write_num! {
        write_u16: u16,
        write_u32: u32,
        write_u64: u64,

        write_i16: i16,
        write_i32: i32,
        write_i64: i64,

        write_f32: f32,
        write_f64: f64,
    }

    pub fn write_str(&mut self, s: &str) {
        self.write(s.as_bytes());
        self.write_u8(0);
    }

    pub fn write_c_str(&mut self, s: &CStr) {
        self.write(s.to_bytes_with_nul());
    }
}

impl<const N: usize> Default for MessageBuilder<N> {
    fn default() -> Self {
        Self::new()
    }
}
