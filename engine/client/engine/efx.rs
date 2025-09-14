use core::ffi::c_int;

use shared::{
    ffi::{
        api::efx::{efx_api_s, TEMPENTITY},
        common::{dlight_s, vec3_t, BEAM},
    },
    str::{AsCStrPtr, ToEngineStr},
};

use crate::{
    entity::TempEntityFlags,
    raw::{RenderFx, RenderMode},
};

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
#[repr(transparent)]
pub struct BeamEntity(u16);

impl BeamEntity {
    /// Creates `BeamEntity` if the given values are valid.
    ///
    /// The `index` is valid if it is less than `0x1000`. The `attachment` is valid if it is less
    /// than `0x10`.
    pub fn new(index: u16, attachment: u16) -> Option<BeamEntity> {
        if index < 0x1000 && attachment < 0x10 {
            Some(unsafe { Self::new_unchecked(index, attachment) })
        } else {
            None
        }
    }

    /// Creates `BeamEntity` without checking whether arguments are valid. This results
    /// in undefined behavior if arguments is not valid.
    ///
    /// # Safety
    ///
    /// * `index` must be less than `0x1000`.
    /// * `attachment` must be less than `0x10`.
    pub unsafe fn new_unchecked(index: u16, attachment: u16) -> BeamEntity {
        Self((attachment << 12) | index)
    }

    /// Creates `BeamEntity` from a raw value.
    pub fn from_bits(bits: u16) -> BeamEntity {
        BeamEntity(bits)
    }

    /// Returns the underlying bits value.
    pub fn bits(&self) -> u16 {
        self.0
    }

    /// Returns the index.
    pub fn index(&self) -> u16 {
        self.0 & 0xfff
    }

    /// Returns the attachment.
    pub fn attachment(&self) -> u16 {
        self.0 >> 12
    }
}

pub struct EfxApi {
    raw: *mut efx_api_s,
}

macro_rules! unwrap {
    ($self:expr, $name:ident) => {
        match $self.raw().$name {
            Some(func) => func,
            None => panic!("efx_api_s.{} is null", stringify!($name)),
        }
    };
}

impl EfxApi {
    pub(super) fn new(raw: *mut efx_api_s) -> Self {
        Self { raw }
    }

    pub fn raw(&self) -> &efx_api_s {
        unsafe { self.raw.as_ref().unwrap() }
    }

    // pub R_AllocParticle: Option<
    //     unsafe extern "C" fn(
    //         callback: Option<unsafe extern "C" fn(particle: *mut particle_s, frametime: f32)>,
    //     ) -> *mut particle_t,
    // >,
    // pub R_BlobExplosion: Option<unsafe extern "C" fn(org: *const f32)>,
    // pub R_Blood:
    //     Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, pcolor: c_int, speed: c_int)>,
    // pub R_BloodSprite: Option<
    //     unsafe extern "C" fn(
    //         org: *const f32,
    //         colorindex: c_int,
    //         modelIndex: c_int,
    //         modelIndex2: c_int,
    //         size: f32,
    //     ),
    // >,
    // pub R_BloodStream:
    //     Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, pcolor: c_int, speed: c_int)>,
    // pub R_BreakModel: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         size: *const f32,
    //         dir: *const f32,
    //         random: f32,
    //         life: f32,
    //         count: c_int,
    //         modelIndex: c_int,
    //         flags: c_char,
    //     ),
    // >,
    // pub R_Bubbles: Option<
    //     unsafe extern "C" fn(
    //         mins: *const f32,
    //         maxs: *const f32,
    //         height: f32,
    //         modelIndex: c_int,
    //         count: c_int,
    //         speed: f32,
    //     ),
    // >,
    // pub R_BubbleTrail: Option<
    //     unsafe extern "C" fn(
    //         start: *const f32,
    //         end: *const f32,
    //         height: f32,
    //         modelIndex: c_int,
    //         count: c_int,
    //         speed: f32,
    //     ),
    // >,

    pub fn bullet_impact_particles(&self, pos: vec3_t) {
        unsafe { unwrap!(self, R_BulletImpactParticles)(pos.as_ptr()) }
    }

    // pub R_EntityParticles: Option<unsafe extern "C" fn(ent: *mut cl_entity_s)>,
    // pub R_Explosion: Option<
    //     unsafe extern "C" fn(pos: *mut f32, model: c_int, scale: f32, framerate: f32, flags: c_int),
    // >,
    // pub R_FizzEffect:
    //     Option<unsafe extern "C" fn(pent: *mut cl_entity_s, modelIndex: c_int, density: c_int)>,
    // pub R_FireField: Option<
    //     unsafe extern "C" fn(
    //         org: *mut f32,
    //         radius: c_int,
    //         modelIndex: c_int,
    //         count: c_int,
    //         flags: c_int,
    //         life: f32,
    //     ),
    // >,
    // pub R_FlickerParticles: Option<unsafe extern "C" fn(org: *const f32)>,
    // pub R_FunnelSprite:
    //     Option<unsafe extern "C" fn(org: *const f32, modelIndex: c_int, reverse: c_int)>,
    // pub R_Implosion:
    //     Option<unsafe extern "C" fn(end: *const f32, radius: f32, count: c_int, life: f32)>,
    // pub R_LargeFunnel: Option<unsafe extern "C" fn(org: *const f32, reverse: c_int)>,
    // pub R_LavaSplash: Option<unsafe extern "C" fn(org: *const f32)>,
    // pub R_MultiGunshot: Option<
    //     unsafe extern "C" fn(
    //         org: *const f32,
    //         dir: *const f32,
    //         noise: *const f32,
    //         count: c_int,
    //         decalCount: c_int,
    //         decalIndices: *mut c_int,
    //     ),
    // >,
    // pub R_MuzzleFlash: Option<unsafe extern "C" fn(pos1: *const f32, type_: c_int)>,
    // pub R_ParticleBox: Option<
    //     unsafe extern "C" fn(
    //         mins: *const f32,
    //         maxs: *const f32,
    //         r: c_uchar,
    //         g: c_uchar,
    //         b: c_uchar,
    //         life: f32,
    //     ),
    // >,
    // pub R_ParticleBurst:
    //     Option<unsafe extern "C" fn(pos: *const f32, size: c_int, color: c_int, life: f32)>,
    // pub R_ParticleExplosion: Option<unsafe extern "C" fn(org: *const f32)>,
    // pub R_ParticleExplosion2:
    //     Option<unsafe extern "C" fn(org: *const f32, colorStart: c_int, colorLength: c_int)>,
    // pub R_ParticleLine: Option<
    //     unsafe extern "C" fn(
    //         start: *const f32,
    //         end: *const f32,
    //         r: c_uchar,
    //         g: c_uchar,
    //         b: c_uchar,
    //         life: f32,
    //     ),
    // >,
    // pub R_PlayerSprites:
    //     Option<unsafe extern "C" fn(client: c_int, modelIndex: c_int, count: c_int, size: c_int)>,
    // pub R_Projectile: Option<
    //     unsafe extern "C" fn(
    //         origin: *const f32,
    //         velocity: *const f32,
    //         modelIndex: c_int,
    //         life: c_int,
    //         owner: c_int,
    //         hitcallback: Option<unsafe extern "C" fn(ent: *mut tempent_s, ptr: *mut pmtrace_s)>,
    //     ),
    // >,
    // pub R_RicochetSound: Option<unsafe extern "C" fn(pos: *const f32)>,
    // pub R_RicochetSprite: Option<
    //     unsafe extern "C" fn(pos: *const f32, pmodel: *mut model_s, duration: f32, scale: f32),
    // >,
    // pub R_RocketFlare: Option<unsafe extern "C" fn(pos: *const f32)>,

    pub fn rocket_trail(&self, mut start: vec3_t, mut end: vec3_t, type_: c_int) {
        let start = start.as_mut_ptr().cast();
        let end = end.as_mut_ptr().cast();
        // FIXME: ffi: why start and end are mutable?
        unsafe { unwrap!(self, R_RocketTrail)(start, end, type_) }
    }

    // pub R_RunParticleEffect:
    //     Option<unsafe extern "C" fn(org: *const f32, dir: *const f32, color: c_int, count: c_int)>,
    // pub R_ShowLine: Option<unsafe extern "C" fn(start: *const f32, end: *const f32)>,

    pub fn spark_effect(
        &self,
        pos: vec3_t,
        count: c_int,
        velocity_min: c_int,
        velocity_max: c_int,
    ) {
        unsafe { unwrap!(self, R_SparkEffect)(pos.as_ptr(), count, velocity_min, velocity_max) }
    }

    pub fn spark_shower(&self, pos: vec3_t) {
        unsafe { unwrap!(self, R_SparkShower)(pos.as_ptr()) }
    }

    // pub R_SparkStreaks: Option<
    //     unsafe extern "C" fn(pos: *const f32, count: c_int, velocityMin: c_int, velocityMax: c_int),
    // >,
    // pub R_Spray: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         dir: *const f32,
    //         modelIndex: c_int,
    //         count: c_int,
    //         speed: c_int,
    //         spread: c_int,
    //         rendermode: c_int,
    //     ),
    // >,
    // pub R_Sprite_Explode:
    //     Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32, flags: c_int)>,
    // pub R_Sprite_Smoke: Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32)>,
    // pub R_Sprite_Spray: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         dir: *const f32,
    //         modelIndex: c_int,
    //         count: c_int,
    //         speed: c_int,
    //         iRand: c_int,
    //     ),
    // >,

    #[allow(clippy::too_many_arguments)]
    pub fn sprite_trail(
        &self,
        type_: c_int,
        mut start: vec3_t,
        mut end: vec3_t,
        model_index: c_int,
        count: c_int,
        life: f32,
        size: f32,
        amplitude: f32,
        renderamt: c_int,
        speed: f32,
    ) {
        unsafe {
            // FIXME: ffi: why start and end are mutable?
            unwrap!(self, R_Sprite_Trail)(
                type_,
                start.as_mut_ptr(),
                end.as_mut_ptr(),
                model_index,
                count,
                life,
                size,
                amplitude,
                renderamt,
                speed,
            )
        }
    }

    // pub R_Sprite_WallPuff: Option<unsafe extern "C" fn(pTemp: *mut TEMPENTITY, scale: f32)>,
    // pub R_StreakSplash: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         dir: *const f32,
    //         color: c_int,
    //         count: c_int,
    //         speed: f32,
    //         velocityMin: c_int,
    //         velocityMax: c_int,
    //     ),
    // >,

    pub fn create_tracer(&self, start: vec3_t, end: vec3_t) {
        unsafe { unwrap!(self, R_TracerEffect)(start.as_ptr(), end.as_ptr()) }
    }

    // pub R_UserTracerParticle: Option<
    //     unsafe extern "C" fn(
    //         org: *mut f32,
    //         vel: *mut f32,
    //         life: f32,
    //         colorIndex: c_int,
    //         length: f32,
    //         deathcontext: c_uchar,
    //         deathfunc: Option<unsafe extern "C" fn(particle: *mut particle_s)>,
    //     ),
    // >,
    // pub R_TracerParticles:
    //     Option<unsafe extern "C" fn(org: *mut f32, vel: *mut f32, life: f32) -> *mut particle_t>,
    // pub R_TeleportSplash: Option<unsafe extern "C" fn(org: *const f32)>,
    // pub R_TempSphereModel: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         speed: f32,
    //         life: f32,
    //         count: c_int,
    //         modelIndex: c_int,
    //     ),
    // >,

    pub fn temp_model(
        &self,
        pos: vec3_t,
        dir: vec3_t,
        angles: vec3_t,
        life: f32,
        model: c_int,
        soundtype: c_int,
    ) -> *mut TEMPENTITY {
        unsafe {
            unwrap!(self, R_TempModel)(
                pos.as_ptr(),
                dir.as_ptr(),
                angles.as_ptr(),
                life,
                model,
                soundtype,
            )
        }
    }

    // pub R_DefaultSprite: Option<
    //     unsafe extern "C" fn(
    //         pos: *const f32,
    //         spriteIndex: c_int,
    //         framerate: f32,
    //     ) -> *mut TEMPENTITY,
    // >,

    #[allow(clippy::too_many_arguments)]
    pub fn temp_sprite(
        &self,
        mut pos: vec3_t,
        dir: vec3_t,
        scale: f32,
        model_index: c_int,
        rendermode: RenderMode,
        renderfx: RenderFx,
        a: f32,
        life: f32,
        flags: TempEntityFlags,
    ) -> *mut TEMPENTITY {
        unsafe {
            // FIXME: ffi: why pos is mutable?
            unwrap!(self, R_TempSprite)(
                pos.as_mut_ptr(),
                dir.as_ptr(),
                scale,
                model_index,
                rendermode as c_int,
                renderfx as c_int,
                a,
                life,
                flags.bits(),
            )
        }
    }

    pub fn draw_decal_index(&self, id: c_int) -> c_int {
        unsafe { unwrap!(self, Draw_DecalIndex)(id) }
    }

    pub fn draw_decal_index_from_name(&self, name: impl ToEngineStr) -> c_int {
        let name = name.to_engine_str();
        unsafe { unwrap!(self, Draw_DecalIndexFromName)(name.as_ptr()) }
    }

    pub fn decal_shoot(
        &self,
        texture_index: c_int,
        entity: c_int,
        model_index: c_int,
        mut position: vec3_t,
        flags: c_int,
    ) {
        unsafe {
            // FIXME: ffi: why position is mutable?
            unwrap!(self, R_DecalShoot)(
                texture_index,
                entity,
                model_index,
                position.as_mut_ptr().cast(),
                flags,
            )
        }
    }

    // pub R_AttachTentToPlayer:
    //     Option<unsafe extern "C" fn(client: c_int, modelIndex: c_int, zoffset: f32, life: f32)>,
    // pub R_KillAttachedTents: Option<unsafe extern "C" fn(client: c_int)>,
    // pub R_BeamCirclePoints: Option<
    //     unsafe extern "C" fn(
    //         type_: c_int,
    //         start: *mut f32,
    //         end: *mut f32,
    //         modelIndex: c_int,
    //         life: f32,
    //         width: f32,
    //         amplitude: f32,
    //         brightness: f32,
    //         speed: f32,
    //         startFrame: c_int,
    //         framerate: f32,
    //         r: f32,
    //         g: f32,
    //         b: f32,
    //     ) -> *mut BEAM,
    // >,

    #[allow(clippy::too_many_arguments)]
    pub fn beam_ent_point(
        &self,
        start_ent: BeamEntity,
        mut end: vec3_t,
        model_index: c_int,
        life: f32,
        width: f32,
        amplitude: f32,
        brightness: f32,
        speed: f32,
        start_frame: c_int,
        framerate: f32,
        color: [f32; 3],
    ) -> *mut BEAM {
        unsafe {
            // FIXME: ffi: why end is mutable?
            unwrap!(self, R_BeamEntPoint)(
                start_ent.bits().into(),
                end.as_mut_ptr(),
                model_index,
                life,
                width,
                amplitude,
                brightness,
                speed,
                start_frame,
                framerate,
                color[0],
                color[1],
                color[2],
            )
        }
    }

    // pub R_BeamEnts: Option<
    //     unsafe extern "C" fn(
    //         startEnt: c_int,
    //         endEnt: c_int,
    //         modelIndex: c_int,
    //         life: f32,
    //         width: f32,
    //         amplitude: f32,
    //         brightness: f32,
    //         speed: f32,
    //         startFrame: c_int,
    //         framerate: f32,
    //         r: f32,
    //         g: f32,
    //         b: f32,
    //     ) -> *mut BEAM,
    // >,
    // pub R_BeamFollow: Option<
    //     unsafe extern "C" fn(
    //         startEnt: c_int,
    //         modelIndex: c_int,
    //         life: f32,
    //         width: f32,
    //         r: f32,
    //         g: f32,
    //         b: f32,
    //         brightness: f32,
    //     ) -> *mut BEAM,
    // >,
    // pub R_BeamKill: Option<unsafe extern "C" fn(deadEntity: c_int)>,
    // pub R_BeamLightning: Option<
    //     unsafe extern "C" fn(
    //         start: *mut f32,
    //         end: *mut f32,
    //         modelIndex: c_int,
    //         life: f32,
    //         width: f32,
    //         amplitude: f32,
    //         brightness: f32,
    //         speed: f32,
    //     ) -> *mut BEAM,
    // >,

    #[allow(clippy::too_many_arguments)]
    pub fn beam_points(
        &self,
        mut start: vec3_t,
        mut end: vec3_t,
        model_ndex: c_int,
        life: f32,
        width: f32,
        amplitude: f32,
        brightness: f32,
        speed: f32,
        start_frame: c_int,
        framerate: f32,
        color: [f32; 3],
    ) -> *mut BEAM {
        unsafe {
            // FIXME: ffi: why start and end are mutable?
            unwrap!(self, R_BeamPoints)(
                start.as_mut_ptr(),
                end.as_mut_ptr(),
                model_ndex,
                life,
                width,
                amplitude,
                brightness,
                speed,
                start_frame,
                framerate,
                color[0],
                color[1],
                color[2],
            )
        }
    }

    // pub R_BeamRing: Option<
    //     unsafe extern "C" fn(
    //         startEnt: c_int,
    //         endEnt: c_int,
    //         modelIndex: c_int,
    //         life: f32,
    //         width: f32,
    //         amplitude: f32,
    //         brightness: f32,
    //         speed: f32,
    //         startFrame: c_int,
    //         framerate: f32,
    //         r: f32,
    //         g: f32,
    //         b: f32,
    //     ) -> *mut BEAM,
    // >,

    pub fn alloc_dlight(&self, key: c_int) -> *mut dlight_s {
        unsafe { unwrap!(self, CL_AllocDlight)(key) }
    }

    // pub CL_AllocElight: Option<unsafe extern "C" fn(key: c_int) -> *mut dlight_t>,
    // pub CL_TempEntAlloc:
    //     Option<unsafe extern "C" fn(org: *const f32, model: *mut model_s) -> *mut TEMPENTITY>,
    // pub CL_TempEntAllocNoModel: Option<unsafe extern "C" fn(org: *const f32) -> *mut TEMPENTITY>,
    // pub CL_TempEntAllocHigh:
    //     Option<unsafe extern "C" fn(org: *const f32, model: *mut model_s) -> *mut TEMPENTITY>,
    // pub CL_TentEntAllocCustom: Option<
    //     unsafe extern "C" fn(
    //         origin: *const f32,
    //         model: *mut model_s,
    //         high: c_int,
    //         callback: Option<
    //             unsafe extern "C" fn(ent: *mut tempent_s, frametime: f32, currenttime: f32),
    //         >,
    //     ) -> *mut TEMPENTITY,
    // >,
    // pub R_GetPackedColor: Option<unsafe extern "C" fn(packed: *mut c_short, color: c_short)>,
    // pub R_LookupColor: Option<unsafe extern "C" fn(r: c_uchar, g: c_uchar, b: c_uchar) -> c_short>,
    // pub R_DecalRemoveAll: Option<unsafe extern "C" fn(textureIndex: c_int)>,
    // pub R_FireCustomDecal: Option<
    //     unsafe extern "C" fn(
    //         textureIndex: c_int,
    //         entity: c_int,
    //         modelIndex: c_int,
    //         position: *mut f32,
    //         flags: c_int,
    //         scale: f32,
    //     ),
    // >,
}
