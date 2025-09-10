use core::{ffi::c_int, mem::MaybeUninit};

use csz::{CStrSlice, CStrThin};

use crate::{
    borrow::{BorrowRef, Ref},
    str::{AsCStrPtr, ToEngineStr},
    utils::cstr_or_none,
};

// TODO: delete me
#[rustfmt::skip]
pub use xash3d_ffi::{
    common::netadrtype_t,
    common::netadr_s,

    api::net::net_api_response_func_t,
    api::net::net_adrlist_s,
    api::net::net_response_s,
    api::net::net_status_s,
    api::net::net_api_s,
};

#[derive(Default)]
struct Borrows {
    addr_to_string: BorrowRef,
}

pub struct NetApi {
    raw: *mut net_api_s,
    borrows: Borrows,
}

impl NetApi {
    pub fn new(raw: *mut net_api_s) -> Self {
        Self {
            raw,
            borrows: Borrows::default(),
        }
    }

    pub fn raw(&self) -> &net_api_s {
        unsafe { self.raw.as_ref().unwrap() }
    }
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw().$name {
            Some(func) => func,
            None => panic!("net_api.{} is null", stringify!($name)),
        }
    };
}

/// Engine API to access network functions.
pub trait EngineNet {
    fn net_api(&self) -> &NetApi;

    fn init_networking(&self) {
        unsafe { unwrap!(self.net_api(), InitNetworking)() }
    }

    fn status(&self) -> net_status_s {
        unsafe {
            let mut status = MaybeUninit::uninit();
            unwrap!(self.net_api(), Status)(status.as_mut_ptr());
            status.assume_init()
        }
    }

    fn send_request(
        &self,
        context: c_int,
        request: c_int,
        flags: c_int,
        timeout: f64,
        remote_address: &netadr_s,
        response: net_api_response_func_t,
    ) {
        let mut remote_address = *remote_address;
        unsafe {
            // FIXME: ffi: why remove_address is mutable?
            unwrap!(self.net_api(), SendRequest)(
                context,
                request,
                flags,
                timeout,
                &mut remote_address,
                response,
            );
        }
    }

    fn cancel_request(&self, context: c_int) {
        unsafe { unwrap!(self.net_api(), CancelRequest)(context) }
    }

    fn cancel_all_requests(&self) {
        unsafe { unwrap!(self.net_api(), CancelAllRequests)() }
    }

    fn addr_to_string_ref(&self, addr: &netadr_s) -> Ref<'_, CStrThin> {
        let mut addr = *addr;
        let net_api = self.net_api();
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            // XXX: uses pfnAdrToString under the hood
            // FIXME: ffi: why addr is mutable?
            let s = unwrap!(net_api, AdrToString)(&mut addr);
            net_api.borrows.addr_to_string.borrow(s as *mut CStrThin)
        }
    }

    fn compare_addr(&self, a: &netadr_s, b: &netadr_s) -> bool {
        let mut a = *a;
        let mut b = *b;
        // FIXME: ffi: why arguments are mutable?
        unsafe { unwrap!(self.net_api(), CompareAdr)(&mut a, &mut b) != 0 }
    }

    fn string_to_addr(&self, s: impl ToEngineStr) -> Option<netadr_s> {
        let s = s.to_engine_str();
        let mut netadr_s = MaybeUninit::uninit();
        let s = s.as_ptr().cast_mut();
        // FIXME: ffi: why string is mutable?
        let res = unsafe { unwrap!(self.net_api(), StringToAdr)(s, netadr_s.as_mut_ptr()) };
        if res != 0 {
            Some(unsafe { netadr_s.assume_init() })
        } else {
            None
        }
    }

    fn value_for_key(&self, str: impl ToEngineStr, key: impl ToEngineStr) -> &CStrThin {
        let str = str.to_engine_str();
        let key = key.to_engine_str();
        let res = unsafe { unwrap!(self.net_api(), ValueForKey)(str.as_ptr(), key.as_ptr()) };
        unsafe { cstr_or_none(res).unwrap() }
    }

    fn remove_key(&self, str: &mut CStrSlice, key: impl ToEngineStr) {
        let key = key.to_engine_str();
        unsafe { unwrap!(self.net_api(), RemoveKey)(str.as_mut_ptr(), key.as_ptr()) }
    }

    fn set_value_for_key(
        &self,
        str: &mut CStrSlice,
        key: impl ToEngineStr,
        value: impl ToEngineStr,
    ) {
        let key = key.to_engine_str();
        let value = value.to_engine_str();
        unsafe {
            unwrap!(self.net_api(), SetValueForKey)(
                str.as_mut_ptr(),
                key.as_ptr(),
                value.as_ptr(),
                str.capacity() as c_int,
            )
        }
    }
}
