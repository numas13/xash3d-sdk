use core::{ffi::c_int, num::NonZeroU8};

use bitflags::bitflags;
use csz::CStrArray;
use xash3d_shared::{
    color::{RGB, RGBA},
    entity::{BeamEntity, EntityIndex},
    ffi::{self, common::vec3_t},
    macros::define_enum_for_primitive,
    render::RenderMode,
};

#[cfg(feature = "save")]
use crate::save::{Restore, Save};

pub use xash3d_shared::user_message::*;

define_enum_for_primitive! {
    #[derive(Copy, Clone, Debug, PartialEq, Eq)]
    pub enum MessageDest: c_int {
        /// Unreliable send to all clients.
        Broadcast(ffi::common::MSG_BROADCAST),
        /// Reliable send to all clients.
        All(ffi::common::MSG_ALL),

        /// Unreliable send to one client.
        One(ffi::common::MSG_ONE_UNRELIABLE),
        /// Reliable send to one client, an entity must not be None.
        OneReliable(ffi::common::MSG_ONE),

        /// Reliable write to the init string.
        Init(ffi::common::MSG_INIT),

        /// Unreliable send to clients potentially visible from origin.
        Pvs(ffi::common::MSG_PVS),
        /// Reliable send to clients potentially visible from origin.
        PvsReliable(ffi::common::MSG_PVS_R),

        /// Unreliable send to clients potentially audible from origin.
        Pas(ffi::common::MSG_PAS),
        /// Reliable send to clients potentially audible from origin.
        PasReliable(ffi::common::MSG_PAS_R),

        /// Reliable send to all spectator proxies.
        Spec(ffi::common::MSG_SPEC),
    }
}

impl MessageDest {
    pub fn is_reliable(&self) -> bool {
        matches!(
            self,
            Self::OneReliable
                | Self::All
                | Self::Init
                | Self::PvsReliable
                | Self::PasReliable
                | Self::Spec
        )
    }

    pub fn is_unreliable(&self) -> bool {
        !self.is_reliable()
    }
}

macro_rules! default_value {
    ($value:expr) => {
        $value
    };
    () => {
        Default::default()
    };
}

macro_rules! define_temp_entity_msg {
    ($( #[$attr:meta] )*
    pub struct $name:ident($te_type:expr) {
        $(
            $( #[$field_attr:meta] )*
            $( if $if:expr; )?
            pub $field:ident: $field_ty:ty $(= $field_default:expr )?
        ),* $(,)?
    }) => {
        $( #[$attr] )*
        #[derive(Copy, Clone, Debug)]
        pub struct $name {
            $(
                $( #[$field_attr] )*
                pub $field: $field_ty
            ),*
        }

        impl Default for $name {
            fn default() -> $name {
                Self {
                    $( $field: default_value!($( $field_default )?) ),*
                }
            }
        }

        impl ServerMessage for $name {
            fn msg_type(_: Option<i32>) -> i32 {
                ffi::common::svc_temp_entity
            }

            fn msg_write_body<T: UserMessageWrite>(&self, writer: &mut T) {
                writer.write_u8($te_type as u8);
                $(
                    $(
                        let cond: fn(&Self) -> bool = $if;
                        if cond(self)
                    )?
                    { self.$field.msg_write(writer) }
                )*
            }
        }
    }
}

macro_rules! define_simple_constructor {
    ($name:ty) => {
        impl $name {
            pub fn new(position: vec3_t) -> Self {
                Self {
                    position: position.into(),
                }
            }
        }
    };
}

define_temp_entity_msg! {
    /// A particle effect with a ricochet sound.
    pub struct GunShot(ffi::common::TE_GUNSHOT) {
        pub position: Coord<vec3_t>,
    }
}
define_simple_constructor!(GunShot);

define_temp_entity_msg! {
    /// Quake1 "tarbaby" explosion with sound.
    pub struct TarExplosion(ffi::common::TE_TAREXPLOSION) {
        pub position: Coord<vec3_t>,
    }
}
define_simple_constructor!(TarExplosion);

define_temp_entity_msg! {
    /// 8 random tracers with gravity, ricochet sprite.
    pub struct Sparks(ffi::common::TE_SPARKS) {
        pub position: Coord<vec3_t>,
    }
}
define_simple_constructor!(Sparks);

define_temp_entity_msg! {
    /// Quake1 lava splash.
    pub struct LavaSplash(ffi::common::TE_LAVASPLASH) {
        pub position: Coord<vec3_t>,
    }
}
define_simple_constructor!(LavaSplash);

define_temp_entity_msg! {
    /// Quake1 teleport splash.
    pub struct Teleport(ffi::common::TE_TELEPORT) {
        pub position: Coord<vec3_t>,
    }
}
define_simple_constructor!(Teleport);

define_temp_entity_msg! {
    /// A beam effect between two points.
    pub struct BeamPoints(ffi::common::TE_BEAMPOINTS) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// A beam effect between a point and an entity.
    pub struct BeamEntPoint(ffi::common::TE_BEAMENTPOINT) {
        pub start: BeamEntity,
        pub end: Coord<vec3_t>,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

bitflags! {
    /// The explosion flags to control performance and aesthetic features.
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct ExplosionFlags: u8 {
        /// Default Half-Life explosion.
        const NONE = ffi::common::TE_EXPLFLAG_NONE as u8;
        /// Sprite will be drawn opaque (ensure that the sprite you send is a non-additive sprite).
        const NOADDITIVE = ffi::common::TE_EXPLFLAG_NOADDITIVE as u8;
        /// Do not render dynamic lights.
        const NODLIGHTS = ffi::common::TE_EXPLFLAG_NODLIGHTS as u8;
        /// Do not play client explosion sound.
        const NOSOUND = ffi::common::TE_EXPLFLAG_NOSOUND as u8;
        /// Do not draw particles.
        const NOPARTICLES = ffi::common::TE_EXPLFLAG_NOPARTICLES as u8;
        /// Sprite will be drawn alpha.
        const DRAWALPHA = ffi::common::TE_EXPLFLAG_DRAWALPHA as u8;
        /// Rotate the sprite randomly.
        const ROTATE = ffi::common::TE_EXPLFLAG_ROTATE as u8;
    }
}

impl_message_value_for_bitflags!(ExplosionFlags, u8, write_u8, read_u8);

define_temp_entity_msg! {
    /// An explosion effect with a sound.
    ///
    /// * Additive sprite
    /// * 2 dynamic lights
    /// * Flickering particles
    /// * An explosion sound
    /// * Move vertically 8 pps
    pub struct Explosion(ffi::common::svc_temp_entity) {
        pub position: Coord<vec3_t>,
        pub sprite_index: u16,
        pub scale: FixedU8 = FixedU8::from_u32(1),
        pub frame_rate: u8,
        pub flags: ExplosionFlags = ExplosionFlags::NONE,
    }
}

define_temp_entity_msg! {
    /// Alphablend sprite, move vertically 30 pps.
    pub struct Smoke(ffi::common::TE_SMOKE) {
        pub position: Coord<vec3_t>,
        pub sprite_index: u16,
        pub scale: FixedU8 = FixedU8::from_u32(1),
        pub frame_rate: u8,
    }
}

define_temp_entity_msg! {
    /// A tracer effect from point to point.
    pub struct Tracer(ffi::common::TE_TRACER) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
    }
}

impl Tracer {
    pub fn new(start: vec3_t, end: vec3_t) -> Self {
        Self {
            start: start.into(),
            end: end.into(),
        }
    }
}

define_temp_entity_msg! {
    /// [BeamPoints] with simplified parameters.
    pub struct Lightning(ffi::common::TE_LIGHTNING) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub sprite_index: u16,
    }
}

define_temp_entity_msg! {
    /// A beam effect between two entities.
    pub struct BeamEnts(ffi::common::TE_BEAMENTS) {
        pub start: BeamEntity,
        pub end: BeamEntity,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Quake1 colormaped (base palette) particle explosion with sound.
    pub struct Explosion2(ffi::common::TE_EXPLOSION2) {
        pub position: Coord<vec3_t>,
        pub start_color: u8,
        pub num_colors: u8,
    }
}

define_temp_entity_msg! {
    /// Decal from the BSP file.
    pub struct BspDecal(ffi::common::TE_BSPDECAL) {
        pub position: Coord<vec3_t>,
        pub texture_index: u16,
        pub entity: EntityIndex,
        if |msg| !msg.entity.is_world_spawn();
        pub model_index: u16,
    }
}

define_temp_entity_msg! {
    /// Tracers moving toward a point.
    pub struct Implosion(ffi::common::TE_IMPLOSION) {
        pub position: Coord<vec3_t>,
        pub radius: u8,
        pub count: u8,
        pub duration: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Line of moving glow sprites with gravity, fadeout, and collisions.
    pub struct SpriteRail(ffi::common::TE_SPRITETRAIL) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub sprite_index: u16,
        pub count: u8 = 1,
        pub duration: FixedU8,
        pub scale: FixedU8 = FixedU8::from_f32(1.0),
        /// Velocity along the vector in 10's units.
        pub velocity: ScaledU8,
        /// Randomness of velocity in 10's units.
        pub velocity_randomness: ScaledU8,
    }
}

define_temp_entity_msg! {
    /// Additive sprite, plays 1 cycle.
    pub struct Sprite(ffi::common::TE_SPRITE) {
        pub position: Coord<vec3_t>,
        pub sprite_index: u16,
        pub scale: FixedU8 = FixedU8::from_u32(1),
        pub brightness: u8 = 255,
    }
}

define_temp_entity_msg! {
    /// A beam with a sprite at the end.
    pub struct BeamSprite(ffi::common::TE_BEAMSPRITE) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub beam_sprite_index: u16,
        pub end_sprite_index: u16,
    }
}

define_temp_entity_msg! {
    /// Screen aligned beam ring. Expands to max radius over lifetime.
    pub struct BeamTorus(ffi::common::TE_BEAMTORUS) {
        pub center: Coord<vec3_t>,
        pub axis_radius: Coord<vec3_t>,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// A disk that expands to max radius over lifetime.
    pub struct BeamDisk(ffi::common::TE_BEAMDISK) {
        pub center: Coord<vec3_t>,
        pub axis_radius: Coord<vec3_t>,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Cylinder that expands to max radius over lifetime.
    pub struct BeamCylinder(ffi::common::TE_BEAMCYLINDER) {
        pub center: Coord<vec3_t>,
        pub axis_radius: Coord<vec3_t>,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Create a line of decaying beam segments until entity stops moving.
    pub struct BeamFollow(ffi::common::TE_BEAMFOLLOW) {
        pub follow: BeamEntity,
        pub sprite_index: u16,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub color: RGBA = RGBA::WHITE,
    }
}

define_temp_entity_msg! {
    pub struct GlowSprite(ffi::common::TE_GLOWSPRITE) {
        pub position: Coord<vec3_t>,
        pub sprite_index: u16,
        pub duration: FixedU8,
        pub scale: FixedU8 = FixedU8::from_f32(1.0),
        pub brightness: u8 = 255,
    }
}

define_temp_entity_msg! {
    /// Connect a beam ring to two entities.
    pub struct BeamRing(ffi::common::TE_BEAMRING) {
        pub start: BeamEntity,
        pub end: BeamEntity,
        pub sprite_index: u16,
        pub start_frame: u8,
        pub frame_rate: FixedU8,
        pub duration: FixedU8,
        pub line_width: FixedU8 = FixedU8::from_f32(1.0),
        pub noise_amplitude: FixedU8<100>,
        pub color: RGBA = RGBA::WHITE,
        pub scroll_speed: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Oriented shower of tracers.
    pub struct StreakSplash(ffi::common::TE_STREAK_SPLASH) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        pub color: u8,
        pub count: u16,
        pub base_speed: u16,
        pub random_velocity: u16,
    }
}

define_temp_entity_msg! {
    /// Dynamic light, effect world, minor entity effect.
    pub struct Dlight(ffi::common::TE_DLIGHT) {
        pub position: Coord<vec3_t>,
        pub radius: ScaledU8,
        pub color: RGB = RGB::WHITE,
        pub duration: FixedU8,
        pub decay_rate: ScaledU8,
    }
}

define_temp_entity_msg! {
    /// Point entity light, no world effect.
    pub struct Elight(ffi::common::TE_ELIGHT) {
        pub entity: BeamEntity,
        pub position: Coord<vec3_t>,
        pub radius: Coord<f32>,
        pub color: RGB = RGB::WHITE,
        pub duration: FixedU8,
        pub decay_rate: Coord<f32>,
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct TextMessageEffect(u8);

impl_message_value_for_newtype!(TextMessageEffect, u8, write_u8, read_u8);

define_temp_entity_msg! {
    pub struct TextMessage(ffi::common::TE_TEXTMESSAGE) {
        pub channel: u8,
        /// Set to `-1` to center.
        pub x: FixedI16<8192> = FixedI16::from_i32(-1),
        /// Set to `-1` to center.
        pub y: FixedI16<8192> = FixedI16::from_i32(-1),
        // Effect:
        //  0 is fade in/fade out
        //  1 is flickery credits
        //  2 is write out (training room)
        pub effect: TextMessageEffect,
        pub text_color: RGBA = RGBA::WHITE,
        pub effect_color: RGBA = RGBA::WHITE,
        pub fade_in: FixedU16,
        pub fade_out: FixedU16,
        pub hold_time: FixedU16,
        /// Time the highlight lags behing the leading text.
        ///
        /// Optional, used if effect is [Self::EFFECT_WRITE_OUT].
        if |msg| msg.effect == Self::EFFECT_WRITE_OUT;
        pub fx_time: FixedU16,
        pub text_message: CStrArray<512>,
    }
}

impl TextMessage {
    pub const EFFECT_FADE_IN_OUT: TextMessageEffect = TextMessageEffect(0);
    pub const EFFECT_FLICKERY: TextMessageEffect = TextMessageEffect(1);
    pub const EFFECT_WRITE_OUT: TextMessageEffect = TextMessageEffect(2);
}

define_temp_entity_msg! {
    pub struct Line(ffi::common::TE_LINE) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub duration: FixedI16<10>,
        pub color: RGB = RGB::WHITE,
    }
}

define_temp_entity_msg! {
    pub struct Box(ffi::common::TE_BOX) {
        pub mins: Coord<vec3_t>,
        pub maxs: Coord<vec3_t>,
        pub duration: FixedI16<10>,
        pub color: RGB = RGB::WHITE,
    }
}

define_temp_entity_msg! {
    /// Kill all beams attached to entity.
    pub struct KillBeam(ffi::common::TE_KILLBEAM) {
        pub entity: BeamEntity,
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    pub struct LargeFunnelFlags: u16 {
        const REVERSE = 1;
    }
}

impl_message_value_for_bitflags!(LargeFunnelFlags, u16, write_u16, read_u16);

define_temp_entity_msg! {
    pub struct LargeFunnel(ffi::common::TE_LARGEFUNNEL) {
        pub position: Coord<vec3_t>,
        pub sprite_index: u16,
        pub flags: LargeFunnelFlags,
    }
}

define_temp_entity_msg! {
    /// Create a particle spray.
    pub struct BloodStream(ffi::common::TE_BLOODSTREAM) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        pub color: u8,
        pub speed: u8,
    }
}

define_temp_entity_msg! {
    /// Create a particle spray.
    pub struct Blood(ffi::common::TE_BLOOD) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        pub color: u8,
        pub speed: u8,
    }
}

define_temp_entity_msg! {
    /// Create a line of particles every 5 units, dies in 30 seconds.
    pub struct ShowLine(ffi::common::TE_SHOWLINE) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
    }
}

/// Create a decal applied to a brush entity (not the world).
#[derive(Copy, Clone, Debug, Default)]
pub struct Decal {
    /// A center of the texture in world.
    pub position: Coord<vec3_t>,
    /// A texture index of precached decal texture name.
    ///
    /// Must be less than 512.
    pub texture_index: u16,
    pub entity: EntityIndex,
}

impl ServerMessage for Decal {
    fn msg_type(_: Option<i32>) -> i32 {
        ffi::common::svc_temp_entity
    }

    fn msg_write_body<T: UserMessageWrite>(&self, writer: &mut T) {
        let mut msg_type = ffi::common::TE_DECAL;
        let mut texture_index = self.texture_index;

        if self.texture_index >= 256 {
            msg_type = ffi::common::TE_DECALHIGH;
            texture_index -= 256;
            debug_assert!(texture_index < 256);
        }

        writer.write_u8(msg_type as u8);
        writer.write_coord_vec3(self.position);
        writer.write_u8(texture_index as u8);
        writer.write_u16(self.entity.to_u16());
    }
}

define_temp_entity_msg! {
    /// Create alpha sprites inside of entity, float upwards.
    pub struct Fizz(ffi::common::TE_FIZZ) {
        pub entity: EntityIndex,
        pub sprite_index: u16,
        pub density: u8,
    }
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
pub struct BounceSound(u8);

impl BounceSound {
    pub const NONE: Self = Self(ffi::common::TE_BOUNCE_NULL as u8);
    pub const SHELL: Self = Self(ffi::common::TE_BOUNCE_SHELL as u8);
    pub const SHOT_SHELL: Self = Self(ffi::common::TE_BOUNCE_SHOTSHELL as u8);

    pub const fn to_u8(self) -> u8 {
        self.0
    }
}

impl_message_value_for_newtype!(BounceSound, u8, write_u8, read_u8);

define_temp_entity_msg! {
    /// Create a moving model that bounces and makes a sound when it hits
    pub struct Model(ffi::common::TE_MODEL) {
        pub start: Coord<vec3_t>,
        pub velocity: Coord<vec3_t>,
        pub initial_yaw: Angle,
        pub model_index: u16,
        pub bounce_sound: BounceSound,
        pub duration: FixedU8,
    }
}

define_temp_entity_msg! {
    /// Create a spherical shower of models, picks from set.
    pub struct ExplodeModel(ffi::common::TE_EXPLODEMODEL) {
        pub start: Coord<vec3_t>,
        pub velocity: Coord<f32>,
        pub model_index: u16,
        pub count: u16,
        pub duration: FixedU8,
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct BreakModelFlags: u8 {
        const TYPEMASK  = ffi::common::BREAK_TYPEMASK as u8;
        const GLASS     = ffi::common::BREAK_GLASS as u8;
        const METAL     = ffi::common::BREAK_METAL as u8;
        const FLESH     = ffi::common::BREAK_FLESH as u8;
        const WOOD      = ffi::common::BREAK_WOOD as u8;
        const SMOKE     = ffi::common::BREAK_SMOKE as u8;
        const TRANS     = ffi::common::BREAK_TRANS as u8;
        const CONCRETE  = ffi::common::BREAK_CONCRETE as u8;
        // what does the BREAK_2 flag do?
        // const BREAK_2   = ffi::common::BREAK_2 as u8;
    }
}

impl_message_value_for_bitflags!(BreakModelFlags, u8, write_u8, read_u8);

define_temp_entity_msg! {
    /// Create a box of models or sprites.
    pub struct BreakModel(ffi::common::TE_BREAKMODEL) {
        pub position: Coord<vec3_t>,
        pub size: Coord<vec3_t>,
        pub velocity: Coord<vec3_t>,
        pub random_velocity: ScaledU8<10>,
        /// Sprite or model index.
        pub model_index: u16,
        pub count: u8,
        pub duration: FixedU8,
        pub flags: BreakModelFlags,
    }
}

define_temp_entity_msg! {
    /// Create a decal and a ricochet sound.
    pub struct GunShotDecal(ffi::common::TE_GUNSHOTDECAL) {
        pub position: Coord<vec3_t>,
        pub entity: EntityIndex,
        pub decal_index: u8,
    }
}

define_temp_entity_msg! {
    /// Create a spray of alpha sprites
    pub struct SpriteSpray(ffi::common::TE_SPRITE_SPRAY) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        pub sprite_index: u16,
        pub count: u8,
        pub speed: u8,
        pub noise: FixedU8<100>,
    }
}

define_temp_entity_msg! {
    /// Create a quick spark sprite and a client ricochet sound.
    pub struct ArmorRicochet(ffi::common::TE_ARMOR_RICOCHET) {
        pub position: Coord<vec3_t>,
        pub scale: FixedU8<10>,
    }
}

define_temp_entity_msg! {
    pub struct PlayerDecal(ffi::common::TE_PLAYERDECAL) {
        pub player_index: NonZeroU8 = NonZeroU8::new(1).unwrap(),
        pub position: Coord<vec3_t>,
        pub entity: EntityIndex,
        pub decal_index: u8,
    }
}

define_temp_entity_msg! {
    /// Create an alpha sprites inside of box, float upwards.
    pub struct Bubbles(ffi::common::TE_BUBBLES) {
        pub mins: Coord<vec3_t>,
        pub maxs: Coord<vec3_t>,
        pub height: Coord<f32>,
        pub model_index: u16,
        pub count: u8,
        pub speed: Coord<f32>,
    }
}

define_temp_entity_msg! {
    /// Create an alpha sprites along a line, float upwards.
    pub struct BubbleTrail(ffi::common::TE_BUBBLETRAIL) {
        pub start: Coord<vec3_t>,
        pub end: Coord<vec3_t>,
        pub height: Coord<f32>,
        pub model_index: u16,
        pub count: u8,
        pub speed: Coord<f32>,
    }
}

define_temp_entity_msg! {
    /// Create a spray of opaque initial sprite that fall, single droplet sprite for 1..2 secs.
    ///
    /// This is a high-priority tent.
    pub struct BloodSprite(ffi::common::TE_BLOODSPRITE) {
        pub position: Coord<vec3_t>,
        pub initial_sprite_index: u16,
        pub droplet_sprite_index: u16,
        pub color: u8,
        pub scale: u8,
    }
}

/// Create a decal applied to the world brush.
#[derive(Copy, Clone, Debug, Default)]
pub struct WorldDecal {
    /// A decal position (center of texture in world).
    pub position: Coord<vec3_t>,
    /// A texture index of precached decal texture name.
    ///
    /// Must be less than 512.
    pub texture_index: u16,
}

impl ServerMessage for WorldDecal {
    fn msg_type(_: Option<i32>) -> i32 {
        ffi::common::svc_temp_entity
    }

    fn msg_write_body<T: UserMessageWrite>(&self, writer: &mut T) {
        let mut msg_type = ffi::common::TE_WORLDDECAL;
        let mut texture_index = self.texture_index;

        if self.texture_index >= 256 {
            msg_type = ffi::common::TE_WORLDDECALHIGH;
            texture_index -= 256;
            debug_assert!(texture_index < 256);
        }

        writer.write_u8(msg_type as u8);
        writer.write_coord_vec3(self.position);
        writer.write_u8(texture_index as u8);
    }
}

define_temp_entity_msg! {
    /// Create a projectile (like a nail).
    ///
    /// This is a high-priority tent.
    pub struct Projectile(ffi::common::TE_PROJECTILE) {
        pub start: Coord<vec3_t>,
        pub velocity: Coord<vec3_t>,
        pub model_index: u16,
        pub duration: u8,
        // Projectile will not collide with the owner.
        //
        // The projectile will hit any client if the owner is `0`.
        pub owner: u8,
    }
}

define_temp_entity_msg! {
    /// Throws a shower of sprites or models.
    pub struct Spray(ffi::common::TE_SPRAY) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        pub model_index: u16,
        pub count: u8,
        pub speed: u8,
        pub noise: u8,
        pub render_mode: RenderMode,
    }
}

define_temp_entity_msg! {
    /// Emit sprites from a player's bounding box (ONLY use for players!).
    pub struct PlayerSprites(ffi::common::TE_PLAYERSPRITES) {
        pub player: u8,
        pub model_index: u16,
        pub count: u8,
        /// Size variance in percentage.
        ///
        /// * 0 = no variance in size
        /// * 10 = 10% variance in size.
        pub size_variance: u8,
    }
}

define_temp_entity_msg! {
    /// Very similar to [LavaSplash].
    pub struct ParticleBurst(ffi::common::TE_PARTICLEBURST) {
        pub start: Coord<vec3_t>,
        pub radius: i16,
        pub color: u8,
        pub duration: FixedU8<10>,
    }
}

bitflags! {
    #[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
    #[repr(transparent)]
    pub struct FireFieldFlags: u8 {
        /// All sprites will drift upwards as they animate.
        const ALL_FLOAT = ffi::common::TEFIRE_FLAG_ALLFLOAT as u8;
        /// Some of the sprites will drift upwards (50% chance).
        const SOME_FLOAT = ffi::common::TEFIRE_FLAG_SOMEFLOAT as u8;
        /// Sprite plays at 15 fps, otherwise plays at whatever rate stretches
        /// the animation over the sprite's duration.
        const LOOP = ffi::common::TEFIRE_FLAG_LOOP as u8;
        /// Sprite is rendered alpha blended at 50% else, opaque.
        const ALPHA = ffi::common::TEFIRE_FLAG_ALPHA as u8;
        /// All fire sprites have same initial Z instead of randomly filling a cube.
        const PLANAR = ffi::common::TEFIRE_FLAG_PLANAR as u8;
        /// Sprite is rendered non-opaque with additive.
        const ADDITIVE = ffi::common::TEFIRE_FLAG_ADDITIVE as u8;
    }
}

impl_message_value_for_bitflags!(FireFieldFlags, u8, write_u8, read_u8);

define_temp_entity_msg! {
    // Create a field of fire.
    pub struct FireField(ffi::common::TE_FIREFIELD) {
        pub start: Coord<vec3_t>,
        /// The fire is made in a square around origin (-radius, -radius to radius, radius).
        pub radius: i16,
        pub model_index: u16,
        pub count: u8,
        pub flags: FireFieldFlags,
        pub duration: FixedU8<10>,
    }
}

define_temp_entity_msg! {
    /// Attaches a temporary entity to a player.
    ///
    /// This is a high-priority temporaty entity.
    pub struct PlayerAttachment(ffi::common::TE_PLAYERATTACHMENT) {
        pub player: u8,
        /// A vertical offset relative to the player's origin.z.
        pub vertical_offset: Coord<f32>,
        pub model_index: u16,
        pub duration: FixedU16<10>,
    }
}

define_temp_entity_msg! {
    /// Will expire all temporary entities attached to a player.
    pub struct KillPlayerAttachments(ffi::common::TE_KILLPLAYERATTACHMENTS) {
        pub player: u8,
    }
}

define_temp_entity_msg! {
    /// A much more compact [GunShot] message.
    ///
    /// This message is used to make a client approximate a 'spray' of gunfire.
    /// Any weapon that fires more than one bullet per frame and fires in
    /// a bit of a spread is a good candidate for MULTIGUNSHOT use.
    ///
    /// <div class="warning">
    ///
    /// This effect makes the client do traces for each bullet, these client traces ignore
    /// entities that have studio models. Traces are 4096 long.
    ///
    /// </div>
    pub struct MultiGunShot(ffi::common::TE_MULTIGUNSHOT) {
        pub start: Coord<vec3_t>,
        pub direction: Coord<vec3_t>,
        /// x noise * 100
        pub x_noise: Coord<f32>,
        /// y noise * 100
        pub y_noise: Coord<f32>,
        pub count: u8,
        pub texture_index: u8,
    }
}

define_temp_entity_msg! {
    /// A larger message than the [Tracer], but allows some customization.
    pub struct UserTracer(ffi::common::TE_USERTRACER) {
        pub start: Coord<vec3_t>,
        pub velocity: Coord<vec3_t>,
        pub duration: FixedU8<10>,
        /// An index into an array of color vectors in the engine.
        pub color: u8,
        pub length: FixedU8<10>,
    }
}

define_user_message! {
    pub struct Intermission = ffi::common::svc_intermission
}

define_user_message! {
    pub struct CdTrack {
        pub track_num: u8,
        pub loop_num: u8,
    } = ffi::common::svc_cdtrack
}

impl CdTrack {
    pub const fn new(track_num: u8, loop_num: u8) -> Self {
        Self {
            track_num,
            loop_num,
        }
    }
}

define_user_message! {
    pub struct WeaponAnimation {
        pub sequence: u8,
        pub weapon_model: u8,
    } = ffi::common::svc_weaponanim
}

#[derive(Copy, Clone, Debug, Default, PartialEq, Eq)]
#[cfg_attr(feature = "save", derive(Save, Restore))]
#[repr(u16)]
pub enum RoomType {
    #[default]
    /// The default, echo-less sound style.
    Normal = 0,
    /// A slightly more closed in sound than default.
    Generic = 1,
    /// Quite similar to Generic, with slightly more ring.
    MetalSmall = 2,
    /// As above, but with slightly longer echo.
    MetalMedium = 3,
    /// As above, but with longer echo.
    MetalLarge = 4,
    /// A drawn out, tinny sound.
    TunnelSmall = 5,
    /// As above, by with more drawn out echo.
    TunnelMedium = 6,
    /// As above, but with a very drawn out echo.
    TunnelLarge = 7,
    /// Similar to Generic, but with more echo.
    ChamberSmall = 8,
    /// As above, but with slightly longer echo.
    ChamberMedium = 9,
    /// As above, but with a long echo.
    ChamberLarge = 10,
    /// Very similar to Generic.
    BrightSmall = 11,
    /// As above, but more open-sounding.
    BrightMedium = 12,
    /// As above, but more open-sounding.
    BrightLarge = 13,
    /// A claustrophobic, muffled sound.
    Water1 = 14,
    /// As above, but with an echo.
    Water2 = 15,
    /// As above, but with a longer, ringing echo.
    Water3 = 16,
    /// Similar to Generic, but with a short echo.
    ConcreteSmall = 17,
    /// As above, but with a longer echo.
    ConcreteMedium = 18,
    /// As above, but with a longer echo.
    ConcreteLarge = 19,
    /// An open sound with a spaced out, ringing echo.
    Big1 = 20,
    /// As above, but with a longer-lingering echo.
    Big2 = 21,
    /// As above, but with a much longer-lingering echo.
    Big3 = 22,
    /// A closed in sound with a fast-ringing echo.
    CavernSmall = 23,
    /// As above, but with a longer-lingering echo.
    CavernMedium = 24,
    /// As above, but with a much longer-lingering echo.
    CavernLarge = 25,
    /// Similar to Generic, but with a sharper sound.
    Weirdo1 = 26,
    /// As above, but with a high, ringing echo.
    Weirdo2 = 27,
    /// As above, but with a strange, high-pitched echo.
    Weirdo3 = 28,
}

impl RoomType {
    pub fn from_raw(raw: u16) -> Option<Self> {
        let ret = match raw {
            0 => Self::Normal,
            1 => Self::Generic,
            2 => Self::MetalSmall,
            3 => Self::MetalMedium,
            4 => Self::MetalLarge,
            5 => Self::TunnelSmall,
            6 => Self::TunnelMedium,
            7 => Self::TunnelLarge,
            8 => Self::ChamberSmall,
            9 => Self::ChamberMedium,
            10 => Self::ChamberLarge,
            11 => Self::BrightSmall,
            12 => Self::BrightMedium,
            13 => Self::BrightLarge,
            14 => Self::Water1,
            15 => Self::Water2,
            16 => Self::Water3,
            17 => Self::ConcreteSmall,
            18 => Self::ConcreteMedium,
            19 => Self::ConcreteLarge,
            20 => Self::Big1,
            21 => Self::Big2,
            22 => Self::Big3,
            23 => Self::CavernSmall,
            24 => Self::CavernMedium,
            25 => Self::CavernLarge,
            26 => Self::Weirdo1,
            27 => Self::Weirdo2,
            28 => Self::Weirdo3,
            _ => return None,
        };
        Some(ret)
    }
}

define_user_message! {
    pub struct SetRoomType {
        pub room_type_raw: u16,
    } = ffi::common::svc_roomtype
}

impl SetRoomType {
    pub const fn new(room_type: RoomType) -> Self {
        Self {
            room_type_raw: room_type as u16,
        }
    }

    pub fn room_type(&self) -> Option<RoomType> {
        RoomType::from_raw(self.room_type_raw)
    }
}

// TODO: define user message for svc_director

/// Take the last path component and convert it to a CStr.
#[doc(hidden)]
#[macro_export]
macro_rules! user_message_name {
    ($path:path, $($tt:tt)+) => {
        $crate::user_message::user_message_name!($($tt)+)
    };
    ($head:ident :: $($tt:tt)+) => {
        $crate::user_message::user_message_name!($($tt)+)
    };
    ($name:ident) => {
        $crate::macros::cstringify!($name)
    };
}
pub use user_message_name;

#[doc(hidden)]
#[macro_export]
macro_rules! register_user_message {
    ($engine:expr, $path:ty = $name:ident $(,)?) => {
        $engine.register_user_message::<$path>($crate::user_message::user_message_name!($name))
    };
    ($engine:expr, $( $tt:tt )+ ) => {
        $engine.register_user_message::<$($tt)+>(
            $crate::user_message::user_message_name!($($tt)+, $($tt)+)
        )
    };
}
#[doc(inline)]
pub use register_user_message;
