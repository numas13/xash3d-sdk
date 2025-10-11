use core::{
    cmp,
    ffi::{c_int, CStr},
    fmt::{self, Write},
    mem,
    num::NonZeroU8,
    str,
};

use alloc::{ffi::CString, string::String};

use csz::{CStrArray, CStrThin};
use xash3d_ffi::common::vec3_t;

use crate::{
    color::{RGB, RGBA},
    entity::{BeamEntity, EntityIndex},
    render::RenderMode,
    str::ToEngineStr,
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum UserMessageError {
    UnexpectedEnd,
    InvalidEnum,
    InvalidNumber,
    InvalidUtf8String,
}

pub trait IntoUserMessageResult {
    fn into_user_message_result(self) -> c_int;
}

impl IntoUserMessageResult for bool {
    fn into_user_message_result(self) -> c_int {
        self as c_int
    }
}

impl IntoUserMessageResult for c_int {
    fn into_user_message_result(self) -> c_int {
        self
    }
}

impl IntoUserMessageResult for Option<()> {
    fn into_user_message_result(self) -> c_int {
        self.is_some() as c_int
    }
}

impl IntoUserMessageResult for Result<(), UserMessageError> {
    fn into_user_message_result(self) -> c_int {
        // TODO: log message error
        self.is_ok() as c_int
    }
}

impl IntoUserMessageResult for Result<bool, UserMessageError> {
    fn into_user_message_result(self) -> c_int {
        // TODO: log message error
        match self {
            Ok(value) => value as c_int,
            Err(_) => 0,
        }
    }
}

/// A user message received from the server dll.
#[derive(Copy, Clone)]
pub struct UserMessageBuffer<'a> {
    name: &'a CStr,
    data: &'a [u8],
    offset: usize,
}

#[allow(dead_code)]
impl<'a> UserMessageBuffer<'a> {
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

    pub fn read<T: UserMessageValue<'a>>(&mut self) -> Result<T, UserMessageError> {
        T::msg_read(self)
    }

    pub fn read_u8(&mut self) -> Result<u8, UserMessageError> {
        self.data
            .get(self.offset)
            .map(|&byte| {
                self.offset += 1;
                byte
            })
            .ok_or(UserMessageError::UnexpectedEnd)
    }

    pub fn read_i8(&mut self) -> Result<i8, UserMessageError> {
        self.read_u8().map(|i| i as i8)
    }

    pub fn read_slice(&mut self, len: usize) -> Result<&'a [u8], UserMessageError> {
        if self.offset + len <= self.data.len() {
            let ret = &self.data[self.offset..self.offset + len];
            self.offset += len;
            Ok(ret)
        } else {
            Err(UserMessageError::UnexpectedEnd)
        }
    }

    pub fn read_array<const N: usize>(&mut self) -> Result<[u8; N], UserMessageError> {
        self.read_slice(N).map(|s| {
            let mut buf = [0; N];
            buf.copy_from_slice(s);
            buf
        })
    }

    pub fn read_u16(&mut self) -> Result<u16, UserMessageError> {
        self.read_array().map(u16::from_le_bytes)
    }

    pub fn read_i16(&mut self) -> Result<i16, UserMessageError> {
        self.read_array().map(i16::from_le_bytes)
    }

    pub fn read_u32(&mut self) -> Result<u32, UserMessageError> {
        self.read_array().map(u32::from_le_bytes)
    }

    pub fn read_i32(&mut self) -> Result<i32, UserMessageError> {
        self.read_array().map(i32::from_le_bytes)
    }

    pub fn read_f32(&mut self) -> Result<f32, UserMessageError> {
        self.read_array().map(f32::from_le_bytes)
    }

    pub fn read_vec3(&mut self) -> Result<vec3_t, UserMessageError> {
        let x = self.read_f32()?;
        let y = self.read_f32()?;
        let z = self.read_f32()?;
        Ok(vec3_t::new(x, y, z))
    }

    pub fn read_coord(&mut self) -> Result<Coord<f32>, UserMessageError> {
        self.read_i16().map(|i| i as f32 * (1.0 / 8.0)).map(Coord)
    }

    pub fn read_coord_vec3(&mut self) -> Result<Coord<vec3_t>, UserMessageError> {
        let x = self.read_coord()?.into();
        let y = self.read_coord()?.into();
        let z = self.read_coord()?.into();
        Ok(vec3_t::new(x, y, z).into())
    }

    pub fn read_angle(&mut self) -> Result<Angle, UserMessageError> {
        self.read_i8()
            .map(|i| i as f32 * (360.0 / 256.0))
            .map(Angle)
    }

    // pub fn read_hires_angle(&mut self) -> Result<f32, MessageError> {
    //     self.read_i16().map(|i| i as f32 * (360.0 / 65536.0))
    // }

    pub fn read_c_str(&mut self) -> Result<&'a CStr, UserMessageError> {
        let s = CStr::from_bytes_until_nul(self.as_slice())
            .map_err(|_| UserMessageError::UnexpectedEnd)?;
        self.offset += s.to_bytes_with_nul().len();
        Ok(s)
    }

    pub fn read_str(&mut self) -> Result<&'a str, UserMessageError> {
        let tail = self.as_slice();
        let len = tail.iter().position(|&i| i == b'\0').unwrap_or(tail.len());
        let end = self.offset + len;

        let s = str::from_utf8(&self.data[self.offset..end])
            .map_err(|_| UserMessageError::InvalidUtf8String)?;
        self.offset = end + 1;
        Ok(s)
    }
}

impl fmt::Display for UserMessageBuffer<'_> {
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

impl fmt::Debug for UserMessageBuffer<'_> {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt::Debug::fmt(self.data, fmt)
    }
}

pub trait UserMessageWrite {
    fn write_u8(&mut self, value: u8);
    fn write_i8(&mut self, value: i8);
    fn write_u16(&mut self, value: u16);
    fn write_i16(&mut self, value: i16);
    fn write_u32(&mut self, value: u32);
    fn write_i32(&mut self, value: i32);
    fn write_f32(&mut self, value: f32);
    fn write_coord(&mut self, coord: Coord<f32>);
    fn write_angle(&mut self, angle: Angle);
    fn write_entity(&mut self, entity: EntityIndex);
    fn write_str(&mut self, str: impl ToEngineStr);

    fn write_vec3(&mut self, v: vec3_t) {
        self.write_f32(v.x);
        self.write_f32(v.y);
        self.write_f32(v.z);
    }

    fn write_coord_vec3(&mut self, coord: Coord<vec3_t>) {
        self.write_coord(Coord(coord.0.x));
        self.write_coord(Coord(coord.0.y));
        self.write_coord(Coord(coord.0.z));
    }
}

pub trait UserMessageValue<'a>: Sized {
    fn msg_size() -> Option<usize> {
        None
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T);

    fn msg_read(msg: &mut UserMessageBuffer<'a>) -> Result<Self, UserMessageError>;
}

impl UserMessageValue<'_> for bool {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(*self as u8);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_u8().map(|i| i != 0)
    }
}

macro_rules! impl_message_value_for_num {
    ($( $ty:ty = $write:ident, $read:ident ;)*) => {
        $(impl UserMessageValue<'_> for $ty {
            fn msg_size() -> Option<usize> {
                Some(mem::size_of::<$ty>())
            }

            fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
                writer.$write(*self);
            }

            fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
                msg.$read()
            }
        })*
    };
}

impl_message_value_for_num! {
    u8 = write_u8, read_u8;
    i8 = write_i8, read_i8;
    u16 = write_u16, read_u16;
    i16 = write_i16, read_i16;
    u32 = write_u32, read_u32;
    i32 = write_i32, read_i32;
}

impl UserMessageValue<'_> for NonZeroU8 {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(self.get());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        NonZeroU8::new(msg.read_u8()?).ok_or(UserMessageError::InvalidNumber)
    }
}

impl UserMessageValue<'_> for RGB {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>() * 3)
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(self.r());
        writer.write_u8(self.g());
        writer.write_u8(self.b());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        let r = msg.read_u8()?;
        let g = msg.read_u8()?;
        let b = msg.read_u8()?;
        Ok(RGB::new(r, g, b))
    }
}

impl UserMessageValue<'_> for RGBA {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>() * 4)
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(self.r());
        writer.write_u8(self.g());
        writer.write_u8(self.b());
        writer.write_u8(self.a());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        let r = msg.read_u8()?;
        let g = msg.read_u8()?;
        let b = msg.read_u8()?;
        let a = msg.read_u8()?;
        Ok(RGBA::new(r, g, b, a))
    }
}

impl UserMessageValue<'_> for EntityIndex {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u16>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_entity(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        let index = msg.read_u16()?;
        Ok(EntityIndex::new(index).unwrap_or(EntityIndex::ZERO))
    }
}

impl UserMessageValue<'_> for BeamEntity {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u16>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u16(self.bits());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_u16().map(Self::from_bits)
    }
}

impl UserMessageValue<'_> for f32 {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u32>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_f32(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_f32()
    }
}

impl UserMessageValue<'_> for vec3_t {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u32>() * 3)
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_vec3(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_vec3()
    }
}

impl UserMessageValue<'_> for RenderMode {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(*self as u8);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        RenderMode::from_raw(msg.read_u8()? as i32).ok_or(UserMessageError::InvalidEnum)
    }
}

impl<'a> UserMessageValue<'a> for &'a CStrThin {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer<'a>) -> Result<Self, UserMessageError> {
        msg.read_c_str().map(|s| s.into())
    }
}

impl<'a> UserMessageValue<'a> for &'a CStr {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer<'a>) -> Result<Self, UserMessageError> {
        msg.read_c_str()
    }
}

impl<const N: usize> UserMessageValue<'_> for CStrArray<N> {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        let bytes = msg.read_c_str()?.to_bytes();
        let len = cmp::min(bytes.len(), N - 1);
        Ok(CStrArray::from_bytes(&bytes[..len]).unwrap())
    }
}

impl UserMessageValue<'_> for CString {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(self.as_c_str());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_c_str().map(|i| i.into())
    }
}

impl<'a> UserMessageValue<'a> for &'a str {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer<'a>) -> Result<Self, UserMessageError> {
        msg.read_str()
    }
}

impl UserMessageValue<'_> for String {
    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_str(self.as_str());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_str().map(|s| s.into())
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedU8<const N: u32 = 10>(u8);

impl<const N: u32> FixedU8<N> {
    pub const ZERO: Self = Self(0);

    pub const fn from_u32(value: u32) -> Self {
        let bits = value * N;
        if bits > u8::MAX as u32 {
            Self(u8::MAX)
        } else {
            Self(bits as u8)
        }
    }

    pub fn from_f32(value: f32) -> Self {
        Self((value * N as f32) as u8)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }
}

impl<const N: u32> From<u32> for FixedU8<N> {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl<const N: u32> From<f32> for FixedU8<N> {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl<const N: u32> UserMessageValue<'_> for FixedU8<N> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(self.bits());
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_u8().map(Self)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedU16<const N: u32 = 256>(u16);

impl<const N: u32> FixedU16<N> {
    pub const ZERO: Self = Self(0);

    pub const fn from_u32(value: u32) -> Self {
        let bits = value * N;
        if bits > u16::MAX as u32 {
            Self(u16::MAX)
        } else {
            Self(bits as u16)
        }
    }

    pub fn from_f32(value: f32) -> Self {
        Self((value * N as f32) as u16)
    }

    pub const fn bits(&self) -> u16 {
        self.0
    }
}

impl<const N: u32> From<u32> for FixedU16<N> {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl<const N: u32> From<f32> for FixedU16<N> {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl<const N: u32> UserMessageValue<'_> for FixedU16<N> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u16>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u16(self.bits())
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_u16().map(Self)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct FixedI16<const N: u32 = 8192>(i16);

impl<const N: u32> FixedI16<N> {
    pub const ZERO: Self = Self(0);

    pub const fn from_i32(value: i32) -> Self {
        let bits = value * N as i32;
        if bits > i16::MAX as i32 {
            Self(i16::MAX)
        } else if bits < i16::MIN as i32 {
            Self(i16::MIN)
        } else {
            Self(bits as i16)
        }
    }

    pub fn from_f32(value: f32) -> Self {
        Self((value * N as f32) as i16)
    }

    pub const fn bits(&self) -> i16 {
        self.0
    }
}

impl<const N: u32> From<i32> for FixedI16<N> {
    fn from(value: i32) -> Self {
        Self::from_i32(value)
    }
}

impl<const N: u32> From<f32> for FixedI16<N> {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl<const N: u32> UserMessageValue<'_> for FixedI16<N> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<i16>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_i16(self.bits())
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_i16().map(Self)
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct ScaledU8<const N: u32 = 10>(u8);

impl<const N: u32> ScaledU8<N> {
    pub const ZERO: Self = Self(0);

    pub const fn from_u32(value: u32) -> Self {
        let bits = value / N;
        if bits > u8::MAX as u32 {
            Self(u8::MAX)
        } else {
            Self(bits as u8)
        }
    }

    pub fn from_f32(value: f32) -> Self {
        Self((value / N as f32) as u8)
    }

    pub const fn bits(&self) -> u8 {
        self.0
    }
}

impl<const N: u32> From<u32> for ScaledU8<N> {
    fn from(value: u32) -> Self {
        Self::from_u32(value)
    }
}

impl<const N: u32> From<f32> for ScaledU8<N> {
    fn from(value: f32) -> Self {
        Self::from_f32(value)
    }
}

impl<const N: u32> UserMessageValue<'_> for ScaledU8<N> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_u8(self.bits())
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_u8().map(Self)
    }
}

/// A floating point value with reduced precision.
#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Coord<T>(pub T);

impl<T> From<T> for Coord<T> {
    fn from(value: T) -> Self {
        Self(value)
    }
}

impl From<Coord<f32>> for f32 {
    fn from(value: Coord<f32>) -> Self {
        value.0
    }
}

impl From<Coord<vec3_t>> for vec3_t {
    fn from(value: Coord<vec3_t>) -> Self {
        value.0
    }
}

impl UserMessageValue<'_> for Coord<f32> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u16>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_coord(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_coord()
    }
}

impl UserMessageValue<'_> for Coord<vec3_t> {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u16>() * 3)
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_coord_vec3(*self)
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        let x = msg.read_coord()?.into();
        let y = msg.read_coord()?.into();
        let z = msg.read_coord()?.into();
        Ok(Self(vec3_t::new(x, y, z)))
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, PartialOrd)]
pub struct Angle(pub f32);

impl Angle {
    pub fn from_degrees(angle: f32) -> Self {
        Self(angle)
    }

    pub fn to_degrees(self) -> f32 {
        self.0
    }
}

impl From<f32> for Angle {
    fn from(value: f32) -> Self {
        Self::from_degrees(value)
    }
}

impl From<Angle> for f32 {
    fn from(value: Angle) -> Self {
        value.to_degrees()
    }
}

impl UserMessageValue<'_> for Angle {
    fn msg_size() -> Option<usize> {
        Some(mem::size_of::<u8>())
    }

    fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
        writer.write_angle(*self);
    }

    fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
        msg.read_angle()
    }
}

pub trait ServerMessage {
    fn msg_type(msg_type: Option<i32>) -> i32;

    fn msg_write_body<T: UserMessageWrite>(&self, writer: &mut T);
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_message_value_for_newtype {
    ($ty:ty, $bits:ty, $write:ident, $read:ident) => {
        impl UserMessageValue<'_> for $ty {
            fn msg_size() -> Option<usize> {
                Some(::core::mem::size_of::<$bits>())
            }

            fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
                writer.$write(self.0)
            }

            fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
                msg.$read().map(Self)
            }
        }
    };
}
#[doc(inline)]
pub use impl_message_value_for_newtype;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_message_value_for_bitflags {
    ($ty:ty, $bits:ty, $write:ident, $read:ident) => {
        impl UserMessageValue<'_> for $ty {
            fn msg_size() -> Option<usize> {
                Some(::core::mem::size_of::<$bits>())
            }

            fn msg_write<T: UserMessageWrite>(&self, writer: &mut T) {
                writer.$write(self.bits())
            }

            fn msg_read(msg: &mut UserMessageBuffer) -> Result<Self, UserMessageError> {
                msg.$read().map(Self::from_bits_retain)
            }
        }
    };
}
#[doc(inline)]
pub use impl_message_value_for_bitflags;

#[doc(hidden)]
#[macro_export]
macro_rules! default_value {
    ($value:expr) => {
        $value
    };
    () => {
        Default::default()
    };
}
pub use default_value;

#[doc(hidden)]
#[macro_export]
macro_rules! impl_user_message_trait {
    ($name:ident { $( $body:tt )* }) => {
        impl $crate::user_message::UserMessageValue<'_> for $name {
            $($body)*
        }
    };
    ($name:ident $(<$lifetime:lifetime>)? { $( $body:tt )* }) => {
        impl $(<$lifetime>)? $crate::user_message::UserMessageValue $(<$lifetime>)?
            for $name $(<$lifetime>)?
        {
            $($body)*
        }
    };
}
pub use impl_user_message_trait;

#[doc(hidden)]
#[macro_export]
macro_rules! define_user_message {
    ($( #[$attr:meta] )*
    $vis:vis struct $name:ident $(<$lifetime:lifetime>)? {
        $(
            $( #[$field_attr:meta] )*
            $field_vis:vis $field:ident: $field_ty:ty $(= $field_default:expr )?
        ),* $(,)?
    }) => {
        $( #[$attr] )*
        #[derive(Copy, Clone, Debug)]
        $vis struct $name $(<$lifetime>)? {
            $(
                $( #[$field_attr] )*
                $field_vis $field: $field_ty
            ),*
        }

        impl $(<$lifetime>)? Default for $name $(<$lifetime>)? {
            fn default() -> Self {
                Self {
                    $( $field: $crate::user_message::default_value!($( $field_default )?) ),*
                }
            }
        }

        $crate::user_message::impl_user_message_trait! {
            $name $(<$lifetime>)? {
                fn msg_size() -> Option<usize> {
                    let mut size = 0;
                    $( size += <$field_ty>::msg_size()?; )*
                    Some(size)
                }

                fn msg_write<T: $crate::user_message::UserMessageWrite>(
                    &self,
                    writer: &mut T,
                ) {
                    use $crate::user_message::UserMessageWrite;
                    $( self.$field.msg_write(writer); )*
                }

                fn msg_read(
                    msg: &mut $crate::user_message::UserMessageBuffer $(<$lifetime>)?,
                ) -> Result<Self, $crate::user_message::UserMessageError> {
                    use $crate::user_message::UserMessageValue;
                    let mut ret = Self::default();
                    $( ret.$field = <$field_ty>::msg_read(msg)?; )*
                    Ok(ret)
                }
            }
        }

        impl $(<$lifetime>)? $crate::user_message::ServerMessage for $name $(<$lifetime>)? {
            fn msg_type(msg_type: Option<i32>) -> i32 {
                use ::core::sync::atomic::{AtomicI32, Ordering};
                static MSG_TYPE: AtomicI32 = AtomicI32::new(0);
                match msg_type {
                    Some(msg_type) => {
                        MSG_TYPE.store(msg_type, Ordering::Relaxed);
                        msg_type
                    }
                    None => MSG_TYPE.load(Ordering::Relaxed),
                }
            }

            fn msg_write_body<T: $crate::user_message::UserMessageWrite>(
                &self,
                writer: &mut T,
            ) {
                use $crate::user_message::UserMessageValue;
                self.msg_write(writer);
            }
        }
    }
}
#[doc(inline)]
pub use define_user_message;

#[cfg(test)]
mod tests {
    #[test]
    fn fixed_u8_10() {
        type FixedU8 = super::FixedU8<10>;

        assert_eq!(FixedU8::from_u32(1).bits(), 10);
        assert_eq!(FixedU8::from_u32(2).bits(), 20);
        assert_eq!(FixedU8::from_u32(3).bits(), 30);
        assert_eq!(FixedU8::from_u32(12).bits(), 120);
        assert_eq!(FixedU8::from_u32(100).bits(), 255);

        assert_eq!(FixedU8::from_f32(0.1).bits(), 1);
        assert_eq!(FixedU8::from_f32(0.2).bits(), 2);
        assert_eq!(FixedU8::from_f32(0.3).bits(), 3);
        assert_eq!(FixedU8::from_f32(1.3).bits(), 13);

        assert_eq!(FixedU8::from_f32(-1.0).bits(), 0);
        assert_eq!(FixedU8::from_f32(100.0).bits(), 255);
    }

    #[test]
    fn fixed_u8_100() {
        type FixedU8 = super::FixedU8<100>;

        assert_eq!(FixedU8::from_u32(1).bits(), 100);
        assert_eq!(FixedU8::from_u32(2).bits(), 200);
        assert_eq!(FixedU8::from_u32(3).bits(), 255);

        assert_eq!(FixedU8::from_f32(0.01).bits(), 1);
        assert_eq!(FixedU8::from_f32(0.02).bits(), 2);
        assert_eq!(FixedU8::from_f32(0.03).bits(), 3);

        assert_eq!(FixedU8::from_f32(0.11).bits(), 11);
        assert_eq!(FixedU8::from_f32(0.22).bits(), 22);
        assert_eq!(FixedU8::from_f32(0.33).bits(), 33);
    }

    #[test]
    fn fixed_u16_256() {
        type FixedU16 = super::FixedU16<256>;

        assert_eq!(FixedU16::from_u32(1).bits(), 256);
        assert_eq!(FixedU16::from_u32(2).bits(), 512);
        assert_eq!(FixedU16::from_u32(3).bits(), 768);

        assert_eq!(FixedU16::from_f32(0.01).bits(), 2);
        assert_eq!(FixedU16::from_f32(0.02).bits(), 5);
        assert_eq!(FixedU16::from_f32(0.03).bits(), 7);

        assert_eq!(FixedU16::from_f32(0.1).bits(), 25);
        assert_eq!(FixedU16::from_f32(0.2).bits(), 51);
        assert_eq!(FixedU16::from_f32(0.3).bits(), 76);

        assert_eq!(FixedU16::from_f32(-1.0).bits(), 0);
        assert_eq!(FixedU16::from_f32(255.999).bits(), 65535);
        assert_eq!(FixedU16::from_f32(256.0).bits(), 65535);
    }

    #[test]
    fn fixed_i16_8192() {
        type FixedI16 = super::FixedI16<8192>;

        assert_eq!(FixedI16::from_i32(-5).bits(), -32768);
        assert_eq!(FixedI16::from_i32(-4).bits(), -32768);
        assert_eq!(FixedI16::from_i32(-3).bits(), -24576);
        assert_eq!(FixedI16::from_i32(-2).bits(), -16384);
        assert_eq!(FixedI16::from_i32(-1).bits(), -8192);

        assert_eq!(FixedI16::from_i32(1).bits(), 8192);
        assert_eq!(FixedI16::from_i32(2).bits(), 16384);
        assert_eq!(FixedI16::from_i32(3).bits(), 24576);
        assert_eq!(FixedI16::from_i32(4).bits(), 32767);
        assert_eq!(FixedI16::from_i32(5).bits(), 32767);

        assert_eq!(FixedI16::from_f32(-0.0004).bits(), -3);
        assert_eq!(FixedI16::from_f32(-0.0003).bits(), -2);
        assert_eq!(FixedI16::from_f32(-0.0002).bits(), -1);
        assert_eq!(FixedI16::from_f32(-0.0001).bits(), 0);

        assert_eq!(FixedI16::from_f32(0.0001).bits(), 0);
        assert_eq!(FixedI16::from_f32(0.0002).bits(), 1);
        assert_eq!(FixedI16::from_f32(0.0003).bits(), 2);
        assert_eq!(FixedI16::from_f32(0.0004).bits(), 3);
    }

    #[test]
    fn scaled_u8_10() {
        type ScaledU8 = super::ScaledU8<10>;

        assert_eq!(ScaledU8::from_u32(1).bits(), 0);
        assert_eq!(ScaledU8::from_u32(9).bits(), 0);
        assert_eq!(ScaledU8::from_u32(10).bits(), 1);
        assert_eq!(ScaledU8::from_u32(123).bits(), 12);
        assert_eq!(ScaledU8::from_u32(1234).bits(), 123);
        assert_eq!(ScaledU8::from_u32(12345).bits(), 255);
    }
}
