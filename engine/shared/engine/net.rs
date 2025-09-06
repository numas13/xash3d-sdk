use core::{
    ffi::{c_char, c_int, c_void},
    mem::MaybeUninit,
};

use csz::{CStrSlice, CStrThin};

use crate::{
    borrow::{BorrowRef, Ref},
    str::{AsCStrPtr, ToEngineStr},
    utils::cstr_or_none,
};

#[allow(non_camel_case_types)]
#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[repr(C)]
pub enum netadrtype_t {
    Loopback = 1,
    Broadcast = 2,
    Ip = 3,
    Ipx = 4,
    BroadcastIpx = 5,
    Ip6 = 6,
    MulticastIp6 = 7,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_s {
    pub netadr_ip_s: netadr_ip_u,
    pub port: u16,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub union netadr_ip_u {
    pub ip6: netadr_ip6_s,
    pub ip: netadr_ip_s,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_ip6_s {
    pub type6: u16,
    pub ip6: [u8; 16],
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C, packed)]
pub struct netadr_ip_s {
    pub type_: u32,
    pub ip4: [u8; 4],
    pub ipx: [u8; 10],
}

#[allow(non_camel_case_types)]
pub type net_api_response_func_t = Option<unsafe extern "C" fn(response: *mut net_response_s)>;

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_adrlist_s {
    pub next: *mut net_adrlist_s,
    pub remote_address: netadr_s,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_response_s {
    pub error: c_int,
    pub context: c_int,
    pub type_: c_int,
    pub remote_address: netadr_s,
    pub ping: f64,
    pub response: *mut c_void,
}

#[allow(non_camel_case_types)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_status_s {
    pub connected: c_int,
    pub local_address: netadr_s,
    pub remote_address: netadr_s,
    pub packet_loss: c_int,
    pub latency: f64,
    pub connection_time: f64,
    pub rate: f64,
}

#[allow(non_camel_case_types)]
#[allow(non_snake_case)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct net_api_s {
    pub InitNetworking: Option<unsafe extern "C" fn()>,
    pub Status: Option<unsafe extern "C" fn(status: *mut net_status_s)>,
    pub SendRequest: Option<
        unsafe extern "C" fn(
            context: c_int,
            request: c_int,
            flags: c_int,
            timeout: f64,
            remote_address: *const netadr_s,
            response: net_api_response_func_t,
        ),
    >,
    pub CancelRequest: Option<unsafe extern "C" fn(context: c_int)>,
    pub CancelAllRequests: Option<unsafe extern "C" fn()>,
    pub AdrToString: Option<unsafe extern "C" fn(a: *const netadr_s) -> *const c_char>,
    pub CompareAdr: Option<unsafe extern "C" fn(a: *const netadr_s, b: *const netadr_s) -> c_int>,
    pub StringToAdr: Option<unsafe extern "C" fn(s: *const c_char, a: *mut netadr_s) -> c_int>,
    pub ValueForKey:
        Option<unsafe extern "C" fn(s: *const c_char, key: *const c_char) -> *const c_char>,
    pub RemoveKey: Option<unsafe extern "C" fn(s: *mut c_char, key: *const c_char)>,
    pub SetValueForKey: Option<
        unsafe extern "C" fn(
            s: *mut c_char,
            key: *const c_char,
            value: *const c_char,
            maxsize: c_int,
        ),
    >,
}

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
        unsafe {
            unwrap!(self.net_api(), SendRequest)(
                context,
                request,
                flags,
                timeout,
                remote_address,
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
        let net_api = self.net_api();
        // SAFETY: The returned string is allocated in a private static buffer
        // in that function. Never returns a null pointer.
        unsafe {
            // XXX: uses pfnAdrToString under the hood
            let s = unwrap!(net_api, AdrToString)(addr);
            net_api.borrows.addr_to_string.borrow(s as *mut CStrThin)
        }
    }

    fn compare_addr(&self, a: &netadr_s, b: &netadr_s) -> bool {
        unsafe { unwrap!(self.net_api(), CompareAdr)(a, b) != 0 }
    }

    fn string_to_addr(&self, s: impl ToEngineStr) -> Option<netadr_s> {
        let s = s.to_engine_str();
        let mut netadr_s = MaybeUninit::uninit();
        let res =
            unsafe { unwrap!(self.net_api(), StringToAdr)(s.as_ptr(), netadr_s.as_mut_ptr()) };
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
