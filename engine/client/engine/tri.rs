use core::ffi::{c_int, c_uchar};

use shared::raw::{model_s, TRICULLSTYLE};

pub const TRI_API_VERSION: c_int = 1;

#[allow(non_camel_case_types)]
pub type triangleapi_s = TriangleApiFunctions;

#[allow(non_snake_case)]
#[derive(Copy, Clone)]
#[repr(C)]
pub struct TriangleApiFunctions {
    pub version: c_int,
    pub RenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    pub Begin: Option<unsafe extern "C" fn(primitiveCode: c_int)>,
    pub End: Option<unsafe extern "C" fn()>,
    pub Color4f: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32)>,
    pub Color4ub: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar)>,
    pub TexCoord2f: Option<unsafe extern "C" fn(u: f32, v: f32)>,
    pub Vertex3fv: Option<unsafe extern "C" fn(worldPnt: *const f32)>,
    pub Vertex3f: Option<unsafe extern "C" fn(x: f32, y: f32, z: f32)>,
    pub Brightness: Option<unsafe extern "C" fn(brightness: f32)>,
    pub CullFace: Option<unsafe extern "C" fn(style: TRICULLSTYLE)>,
    pub SpriteTexture:
        Option<unsafe extern "C" fn(pSpriteModel: *mut model_s, frame: c_int) -> c_int>,
    pub WorldToScreen: Option<unsafe extern "C" fn(world: *const f32, screen: *mut f32) -> c_int>,
    pub Fog: Option<
        unsafe extern "C" fn(flFogColor: *mut [f32; 3usize], flStart: f32, flEnd: f32, bOn: c_int),
    >,
    pub ScreenToWorld: Option<unsafe extern "C" fn(screen: *const f32, world: *mut f32)>,
    pub GetMatrix: Option<unsafe extern "C" fn(pname: c_int, matrix: *mut f32)>,
    pub BoxInPVS: Option<unsafe extern "C" fn(mins: *mut f32, maxs: *mut f32) -> c_int>,
    pub LightAtPoint: Option<unsafe extern "C" fn(pos: *mut f32, value: *mut f32)>,
    pub Color4fRendermode:
        Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32, rendermode: c_int)>,
    pub FogParams: Option<unsafe extern "C" fn(flDensity: f32, iFogSkybox: c_int)>,
}

pub struct TriangleApi {
    raw: *mut TriangleApiFunctions,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw().$name {
            Some(func) => func,
            None => panic!("triangleapi_s.{} is null", stringify!($name)),
        }
    };
}

impl TriangleApi {
    pub(super) fn new(raw: *mut TriangleApiFunctions) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &TriangleApiFunctions {
        unsafe { self.raw.as_ref().unwrap() }
    }

    pub fn version(&self) -> c_int {
        self.raw().version
    }

    // pub RenderMode: Option<unsafe extern "C" fn(mode: c_int)>,
    // pub Begin: Option<unsafe extern "C" fn(primitiveCode: c_int)>,

    pub fn end(&self) {
        unsafe { unwrap!(self, End)() }
    }

    // pub Color4f: Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32)>,
    // pub Color4ub: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar, a: c_uchar)>,
    // pub TexCoord2f: Option<unsafe extern "C" fn(u: f32, v: f32)>,
    // pub Vertex3fv: Option<unsafe extern "C" fn(worldPnt: *const f32)>,
    // pub Vertex3f: Option<unsafe extern "C" fn(x: f32, y: f32, z: f32)>,
    // pub Brightness: Option<unsafe extern "C" fn(brightness: f32)>,
    // pub CullFace: Option<unsafe extern "C" fn(style: TRICULLSTYLE)>,
    // pub SpriteTexture:
    //     Option<unsafe extern "C" fn(pSpriteModel: *mut model_s, frame: c_int) -> c_int>,
    // pub WorldToScreen: Option<unsafe extern "C" fn(world: *const f32, screen: *mut f32) -> c_int>,
    // pub Fog: Option<
    //     unsafe extern "C" fn(flFogColor: *mut [f32; 3usize], flStart: f32, flEnd: f32, bOn: c_int),
    // >,
    // pub ScreenToWorld: Option<unsafe extern "C" fn(screen: *const f32, world: *mut f32)>,
    // pub GetMatrix: Option<unsafe extern "C" fn(pname: c_int, matrix: *mut f32)>,
    // pub BoxInPVS: Option<unsafe extern "C" fn(mins: *mut f32, maxs: *mut f32) -> c_int>,
    // pub LightAtPoint: Option<unsafe extern "C" fn(pos: *mut f32, value: *mut f32)>,
    // pub Color4fRendermode:
    //     Option<unsafe extern "C" fn(r: f32, g: f32, b: f32, a: f32, rendermode: c_int)>,
    // pub FogParams: Option<unsafe extern "C" fn(flDensity: f32, iFogSkybox: c_int)>,
}
