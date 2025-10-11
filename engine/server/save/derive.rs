use alloc::{ffi::CString, string::String, vec::Vec};
use xash3d_shared::{entity::EntityIndex, sound::Attenuation};

use crate::{entity::UseType, str::MapString, time::MapTime};

use super::*;

pub use xash3d_server_derive::{Restore, Save};

pub trait Save {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()>;
}

pub trait Restore {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()>;
}

pub trait RestoreField: Restore {
    fn restore_field(
        &mut self,
        state: &RestoreState,
        cur: &mut Cursor,
        name: &CStr,
    ) -> SaveResult<bool>;
}

pub trait OnRestore {
    fn on_restore(&mut self);
}

macro_rules! impl_save_restore_for_num {
    ($( $ty:ty = $write:ident, $read:ident; )*) => {
        $(
            impl Save for $ty {
                fn save(
                    &self,
                    _: &mut SaveState,
                    cur: &mut CursorMut,
                ) -> SaveResult<()> {
                    cur.$write(*self)?;
                    Ok(())
                }
            }

            impl Restore for $ty {
                fn restore(
                    &mut self,
                    _: &RestoreState,
                    cur: &mut Cursor,
                ) -> SaveResult<()> {
                    *self = cur.$read()?;
                    Ok(())
                }
            }
        )*
    };
}

impl_save_restore_for_num! {
    bool = write_bool, read_bool;

    u8 = write_leb_u8, read_leb_u8;
    i8 = write_leb_i8, read_leb_i8;

    u16 = write_leb_u16, read_leb_u16;
    i16 = write_leb_i16, read_leb_i16;

    u32 = write_leb_u32, read_leb_u32;
    i32 = write_leb_i32, read_leb_i32;

    u64 = write_leb_u64, read_leb_u64;
    i64 = write_leb_i64, read_leb_i64;

    u128 = write_leb_u128, read_leb_u128;
    i128 = write_leb_i128, read_leb_i128;

    usize = write_leb_usize, read_leb_usize;
    isize = write_leb_isize, read_leb_isize;

    f32 = write_f32, read_f32;
    f64 = write_f64_le, read_f64_le;
}

impl Save for MapTime {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_f32(self.as_secs_f32() - state.time())?;
        Ok(())
    }
}

impl Restore for MapTime {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        *self = MapTime::from_secs_f32(cur.read_f32()? + state.time());
        Ok(())
    }
}

impl Save for Option<MapString> {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        let bytes = self.as_ref().map_or(&[0][..], |s| s.to_bytes_with_nul());
        cur.write_bytes_with_size(bytes)?;
        Ok(())
    }
}

impl Restore for Option<MapString> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        if !bytes.is_empty() {
            let s = CStr::from_bytes_with_nul(bytes).map_err(|_| SaveError::InvalidString)?;
            *self = Some(state.engine().new_map_string(s));
        } else {
            *self = None;
        }
        Ok(())
    }
}

impl<T: Save> Save for Option<T> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match self {
            None => {
                cur.write_u8(0)?;
            }
            Some(value) => {
                cur.write_u8(1)?;
                value.save(state, cur)?;
            }
        }
        Ok(())
    }
}

impl<T: Restore + Default> Restore for Option<T> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        match cur.read_u8()? {
            0 => {
                *self = None;
            }
            _ => {
                let mut value = T::default();
                value.restore(state, cur)?;
                *self = Some(value);
            }
        }
        Ok(())
    }
}

impl<T: Save, E: Save> Save for Result<T, E> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match self {
            Err(value) => {
                cur.write_u8(0)?;
                value.save(state, cur)?;
            }
            Ok(value) => {
                cur.write_u8(1)?;
                value.save(state, cur)?;
            }
        }
        Ok(())
    }
}

impl<T: Restore + Default, E: Restore + Default> Restore for Result<T, E> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        match cur.read_u8()? {
            0 => {
                let mut value = E::default();
                value.restore(state, cur)?;
                *self = Err(value);
            }
            _ => {
                let mut value = T::default();
                value.restore(state, cur)?;
                *self = Ok(value);
            }
        }
        Ok(())
    }
}

impl<const N: usize> Save for CStrArray<N> {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.to_bytes())?;
        Ok(())
    }
}

impl<const N: usize> Restore for CStrArray<N> {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        self.cursor()
            .write_bytes(bytes)
            .map_err(|_| SaveError::InvalidString)
    }
}

impl Save for CString {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.to_bytes())?;
        Ok(())
    }
}

impl Restore for CString {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        *self = CString::new(bytes).map_err(|_| SaveError::InvalidString)?;
        Ok(())
    }
}

impl Save for String {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_bytes_with_size(self.as_bytes())?;
        Ok(())
    }
}

impl Restore for String {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let bytes = cur.read_bytes_with_size()?;
        let s = str::from_utf8(bytes).map_err(|_| SaveError::InvalidString)?;
        self.push_str(s);
        Ok(())
    }
}

impl<T: Save> Save for Vec<T> {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        let len_offset = cur.skip(mem::size_of::<u16>())?;
        for i in self {
            i.save(state, cur)?;
        }
        let size = cur.offset() - len_offset - mem::size_of::<u16>();
        let size = size.try_into().map_err(|_| SaveError::SizeOverflow)?;
        cur.write_at(len_offset, |cur| {
            cur.write_u16_le(size)?;
            Ok(())
        })
    }
}

impl<T: Restore + Default> Restore for Vec<T> {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        self.clear();
        let len = cur.read_u16_le()?.into();
        self.reserve(len);
        for _ in 0..len {
            let mut value = T::default();
            value.restore(state, cur)?;
            self.push(value);
        }
        Ok(())
    }
}

impl Save for Attenuation {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_f32((*self).into())?;
        Ok(())
    }
}

impl Restore for Attenuation {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        *self = cur.read_f32()?.into();
        Ok(())
    }
}

impl Save for UseType {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match self {
            Self::Off => cur.write_u8(0)?,
            Self::On => cur.write_u8(1)?,
            Self::Set => cur.write_u8(2)?,
            Self::Toggle => cur.write_u8(3)?,
        };
        Ok(())
    }
}

impl Restore for UseType {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        *self = match cur.read_u8()? {
            0 => Self::Off,
            1 => Self::On,
            2 => Self::Set,
            3 => Self::Toggle,
            _ => return Err(SaveError::InvalidEnum),
        };
        Ok(())
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! impl_save_restore_for_bitflags {
    ($ty:ty) => {
        impl Save for $ty {
            fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
                self.bits().save(state, cur)
            }
        }

        impl Restore for $ty {
            fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
                let mut bits = self.bits();
                bits.restore(state, cur)?;
                *self = <$ty>::from_bits_retain(bits);
                Ok(())
            }
        }
    };
}
#[doc(inline)]
pub use impl_save_restore_for_bitflags;

impl_save_restore_for_bitflags!(xash3d_shared::entity::DamageFlags);

impl Save for EntityIndex {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_leb_u16(self.to_u16())?;
        Ok(())
    }
}

impl Restore for EntityIndex {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        let index = cur.read_leb_u16()?;
        *self = EntityIndex::new(index).ok_or(SaveError::InvalidNumber)?;
        Ok(())
    }
}

impl Save for vec3_t {
    fn save(&self, _: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        cur.write_f32(self.x())?;
        cur.write_f32(self.y())?;
        cur.write_f32(self.z())?;
        Ok(())
    }
}

impl Restore for vec3_t {
    fn restore(&mut self, _: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        self.set_x(cur.read_f32()?);
        self.set_y(cur.read_f32()?);
        self.set_z(cur.read_f32()?);
        Ok(())
    }
}

impl Save for PositionVector {
    fn save(&self, state: &mut SaveState, cur: &mut CursorMut) -> SaveResult<()> {
        match state.use_landmark_offset() {
            Some(offset) => (self.0 - offset).save(state, cur),
            None => self.0.save(state, cur),
        }
    }
}

impl Restore for PositionVector {
    fn restore(&mut self, state: &RestoreState, cur: &mut Cursor) -> SaveResult<()> {
        self.0.restore(state, cur)?;
        if let Some(offset) = state.use_landmark_offset() {
            self.0 += offset;
        }
        Ok(())
    }
}
