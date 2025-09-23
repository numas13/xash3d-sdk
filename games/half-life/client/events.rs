mod crossbow;
mod crowbar;
mod egon;
mod gauss;
mod glock;
mod hgun;
mod mp5;
mod python;
mod rpg;
mod shotgun;
mod snark;
mod train;
mod tripmine;

use core::ffi::{c_int, CStr};

use csz::CStrArray;
use res::valve::{self, sound};
use xash3d_client::{
    consts::{EFLAG_FLESH_SOUND, MAX_PLAYERS, PM_NORMAL, SOLID_BSP},
    engine::{event::event_args_s, ClientEngineRef},
    entity::{Effects, MoveType},
    ffi::{
        common::{pmtrace_s, vec3_t},
        player_move::physent_s,
    },
    macros::hook_event,
    math::AngleVectorsAll,
    prelude::*,
    render::RenderMode,
    sound::Attenuation,
};
use xash3d_player_move as pm;

use crate::export::events_mut;

const DEFAULT_VIEWHEIGHT: f32 = 28.0;
const VEC_DUCK_VIEW: f32 = 12.0;

#[allow(dead_code)]
#[derive(Copy, Clone, PartialEq, Eq)]
#[repr(C)]
enum Bullet {
    None = 0,
    Player9mm,
    PlayerMp5,
    Player357,
    PlayerBuckshot,
    PlayerCrowbar,

    Monster9mm,
    MonsterMp5,
    Monster12mm,
}

pub struct Events {
    engine: ClientEngineRef,
    swing: u32,
    tracer_count: [c_int; MAX_PLAYERS],
    utils: Utils,
}

impl Events {
    pub fn new(engine: ClientEngineRef) -> Self {
        // FIXME: force cl_lw to 0.0 because it is not implemented
        engine.set_cvar(c"cl_lw", false);

        macro_rules! hook {
            ($engine:expr; $($event:expr => $func:ident),* $(,)?) => (
                $(hook_event!($engine, $event, |args| events_mut().$func(args));)*
            );
        }

        hook! {
            engine;
            valve::events::GLOCK1       => fire_glock1,
            valve::events::GLOCK2       => fire_glock2,
            valve::events::SHOTGUN1     => fire_shotgun_single,
            valve::events::SHOTGUN2     => fire_shotgun_double,
            valve::events::MP5          => fire_mp5,
            valve::events::MP52         => fire_mp5_2,
            valve::events::PYTHON       => fire_python,
            valve::events::GAUSS        => fire_gauss,
            valve::events::GAUSSSPIN    => spin_gauss,
            valve::events::TRAIN        => train_pitch_adjust,
            valve::events::CROWBAR      => crowbar,
            valve::events::CROSSBOW1    => fire_crossbow,
            valve::events::CROSSBOW2    => fire_crossbow2,
            valve::events::RPG          => fire_rpg,
            valve::events::EGON_FIRE    => fire_egon,
            valve::events::EGON_STOP    => stop_egon,
            valve::events::FIREHORNET   => fire_hornet_gun,
            valve::events::TRIPFIRE     => fire_tripmine,
            valve::events::SNARKFIRE    => fire_snark,
        }

        Self {
            engine,
            swing: 0,
            tracer_count: [0; MAX_PLAYERS],
            utils: Utils { engine },
        }
    }
}

struct ShellInfo {
    origin: vec3_t,
    velocity: vec3_t,
}

struct Utils {
    engine: ClientEngineRef,
}

impl Utils {
    fn is_player(&self, idx: c_int) -> bool {
        idx >= 1 && idx <= self.engine.get_max_clients()
    }

    fn is_local(&self, idx: c_int) -> bool {
        self.engine.event_api().is_local(idx - 1)
    }

    fn muzzle_flash(&self) {
        let ent = unsafe { &mut *self.engine.get_view_entity() };
        ent.curstate.effects |= Effects::MUZZLEFLASH.bits();
    }

    fn get_player_view_height(&self, args: &event_args_s) -> vec3_t {
        if self.is_player(args.entindex) {
            if self.is_local(args.entindex) {
                return self.engine.event_api().local_player_view_height();
            } else if args.ducking == 1 {
                return vec3_t::new(0.0, 0.0, VEC_DUCK_VIEW);
            }
        }
        vec3_t::new(0.0, 0.0, DEFAULT_VIEWHEIGHT)
    }

    #[allow(clippy::too_many_arguments)]
    fn get_default_shell_info(
        &self,
        args: &mut event_args_s,
        origin: vec3_t,
        velocity: vec3_t,
        av: AngleVectorsAll,
        forward_scale: f32,
        up_scale: f32,
        right_scale: f32,
    ) -> ShellInfo {
        let AngleVectorsAll { forward, right, up } = av;
        let view_ofs = self.get_player_view_height(args);

        let engine = self.engine;
        let r = engine.random_float(50.0, 70.0);
        let u = engine.random_float(100.0, 150.0);

        let shell_origin =
            origin + view_ofs + up * up_scale + forward * forward_scale + right * right_scale;
        let shell_velocity = velocity + right * r + up * u * forward * 25.0;

        ShellInfo {
            origin: shell_origin,
            velocity: shell_velocity,
        }
    }

    fn eject_brass(
        &self,
        origin: vec3_t,
        velocity: vec3_t,
        rotation: f32,
        model: c_int,
        soundtype: c_int,
    ) {
        let endpos = vec3_t::new(0.0, 0.0, rotation);
        self.engine
            .efx_api()
            .temp_model(origin, velocity, endpos, 2.5, model, soundtype);
    }

    fn get_gun_position(&self, args: &event_args_s, origin: vec3_t) -> vec3_t {
        origin + self.get_player_view_height(args)
    }

    fn play_texture_sound(
        &self,
        _idx: c_int,
        tr: &pmtrace_s,
        src: vec3_t,
        end: vec3_t,
        bullet: Bullet,
    ) -> f32 {
        let engine = self.engine;
        let ev = engine.event_api();

        let mut ch_texture_type = 0;

        let entity = ev.index_from_trace(tr);
        if entity == 0 {
            if let Some(texture_name) = ev.trace_texture(tr.ent, src, end) {
                let name = pm::strip_texture_prefix(texture_name.to_bytes());
                let name = CStrArray::<128>::from_bytes(name).unwrap();
                ch_texture_type = pm::find_texture_type(&name)
            }
        } else {
            let cl_entity = engine.get_entity_by_index(entity);
            if !cl_entity.is_null() {
                let cl_entity = unsafe { &*cl_entity };
                if cl_entity.curstate.eflags & EFLAG_FLESH_SOUND as u8 != 0 {
                    ch_texture_type = pm::CHAR_TEX_FLESH;
                }
            }
        }

        let fvol;
        let fvolbar;
        let samples: &[&CStr];
        let mut fattn = Attenuation::NORM;

        match ch_texture_type {
            pm::CHAR_TEX_METAL => {
                fvol = 0.9;
                fvolbar = 0.3;
                samples = &[sound::player::PL_METAL1, sound::player::PL_METAL2];
            }
            pm::CHAR_TEX_DIRT => {
                fvol = 0.9;
                fvolbar = 0.1;
                samples = &[
                    sound::player::PL_DIRT1,
                    sound::player::PL_DIRT2,
                    sound::player::PL_DIRT3,
                ];
            }
            pm::CHAR_TEX_VENT => {
                fvol = 0.5;
                fvolbar = 0.3;
                samples = &[sound::player::PL_DUCT1, sound::player::PL_DUCT2];
            }
            pm::CHAR_TEX_GRATE => {
                fvol = 0.9;
                fvolbar = 0.5;
                samples = &[sound::player::PL_GRATE1, sound::player::PL_GRATE4];
            }
            pm::CHAR_TEX_TILE => {
                fvol = 0.8;
                fvolbar = 0.2;
                samples = &[
                    sound::player::PL_TILE1,
                    sound::player::PL_TILE3,
                    sound::player::PL_TILE2,
                    sound::player::PL_TILE4,
                ];
            }
            pm::CHAR_TEX_SLOSH => {
                fvol = 0.9;
                fvolbar = 0.0;
                samples = &[
                    sound::player::PL_SLOSH1,
                    sound::player::PL_SLOSH3,
                    sound::player::PL_SLOSH2,
                    sound::player::PL_SLOSH4,
                ];
            }
            pm::CHAR_TEX_WOOD => {
                fvol = 0.9;
                fvolbar = 0.2;
                samples = &[
                    sound::debris::WOOD1,
                    sound::debris::WOOD2,
                    sound::debris::WOOD3,
                ];
            }
            pm::CHAR_TEX_GLASS | pm::CHAR_TEX_COMPUTER => {
                fvol = 0.8;
                fvolbar = 0.2;
                samples = &[
                    sound::debris::GLASS1,
                    sound::debris::GLASS1,
                    sound::debris::GLASS2,
                    sound::debris::GLASS3,
                ];
            }
            pm::CHAR_TEX_FLESH => {
                if bullet == Bullet::PlayerCrowbar {
                    return 0.0;
                }

                fvol = 1.0;
                fvolbar = 0.2;
                samples = &[sound::weapons::BULLET_HIT1, sound::weapons::BULLET_HIT2];
                fattn = Attenuation::from(1.0);
            }
            _ => {
                fvol = 0.9;
                fvolbar = 0.6;
                samples = &[sound::player::PL_STEP1, sound::player::PL_STEP2];
            }
        }

        let sample = samples[engine.random_int(0, samples.len() as c_int - 1) as usize];
        ev.build_sound_at(tr.endpos)
            .entity(0)
            .channel_static()
            .volume(fvol)
            .attenuation(fattn)
            .pitch(96 + engine.random_int(0, 0xf))
            .play(sample);

        fvolbar
    }

    fn damage_decal(&self, pe: &physent_s) -> &'static CStr {
        if pe.classnumber == 1 {
            match self.engine.random_int(0, 2) {
                0 => c"{break1",
                1 => c"{break2",
                _ => c"{break3",
            }
        } else if pe.rendermode != RenderMode::Normal as c_int {
            c"{bproof1"
        } else {
            match self.engine.random_int(0, 4) {
                0 => c"{shot1",
                1 => c"{shot2",
                2 => c"{shot3",
                3 => c"{shot4",
                _ => c"{shot5",
            }
        }
    }

    fn gunshot_decal_trace(&self, tr: &pmtrace_s, decal_name: Option<&CStr>) {
        let engine = self.engine;
        let ev = engine.event_api();
        let efx = engine.efx_api();
        efx.bullet_impact_particles(tr.endpos);

        let rand = engine.random_int(0, 0x7fff);
        if rand < 0x7fff / 2 {
            let samples = [
                sound::weapons::RIC1,
                sound::weapons::RIC2,
                sound::weapons::RIC3,
                sound::weapons::RIC4,
                sound::weapons::RIC5,
            ];
            ev.build_sound_at(tr.endpos)
                .entity(-1)
                .play(samples[(rand % 5) as usize]);
        }

        let Some(decal_name) = decal_name else { return };
        let Some(pe) = ev.get_phys_ent(tr.ent) else {
            return;
        };

        if (pe.solid == SOLID_BSP || pe.movetype == MoveType::PushStep.into())
            && engine.get_cvar_float(c"r_decals") != 0.0
        {
            let index = efx.draw_decal_index_from_name(decal_name);
            let texture_index = efx.draw_decal_index(index);
            let ent = ev.index_from_trace(tr);
            efx.decal_shoot(texture_index, ent, 0, tr.endpos, 0);
        }
    }

    fn decal_gunshot(&self, tr: &pmtrace_s, _bullet: Bullet) {
        let ev = self.engine.event_api();

        let Some(pe) = ev.get_phys_ent(tr.ent) else {
            return;
        };
        if pe.solid == SOLID_BSP {
            self.gunshot_decal_trace(tr, Some(self.damage_decal(pe)));
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn check_tracer(
        &self,
        idx: c_int,
        src: vec3_t,
        end: vec3_t,
        forward: vec3_t,
        right: vec3_t,
        _bullet: Bullet,
        (freq, count): (c_int, &mut c_int),
    ) -> bool {
        if freq == 0 {
            return false;
        }

        *count = count.wrapping_add(1);
        if *count % freq != 0 {
            return false;
        }

        let mut tracer_src = src;
        if self.is_player(idx) {
            let offset = vec3_t::new(0.0, 0.0, -4.0);
            tracer_src += offset + right * 2.0 + forward * 16.0;
        }

        self.engine.efx_api().create_tracer(tracer_src, end);

        freq != 1
    }

    #[allow(clippy::too_many_arguments)]
    fn fire_bullets(
        &self,
        idx: c_int,
        av: AngleVectorsAll,
        shots: c_int,
        src: vec3_t,
        dir_shooting: vec3_t,
        distance: f32,
        bullet: Bullet,
        mut tracer: Option<(c_int, &mut c_int)>,
        spread: (f32, f32),
    ) {
        let AngleVectorsAll { forward, right, up } = av;
        let engine = self.engine;
        let ev = engine.event_api();

        for _ in 0..shots {
            let dir = match bullet {
                Bullet::PlayerBuckshot => loop {
                    let x = engine.random_float(-0.5, 0.5) + engine.random_float(-0.5, 0.5);
                    let y = engine.random_float(-0.5, 0.5) + engine.random_float(-0.5, 0.5);
                    let z = x * x + y * y;
                    if z > 1.0 {
                        break dir_shooting + right * spread.0 * x + up * spread.1 * y;
                    }
                },
                _ => dir_shooting + right * spread.0 + up * spread.1,
            };
            let end = src + dir * distance;

            ev.setup_player_predication(false, true);
            let pm_states = ev.push_pm_states();

            ev.set_solid_players(idx - 1);
            ev.set_trace_hull(2);
            let tr = ev.player_trace(src, end, PM_NORMAL, -1);
            let tracer = match &mut tracer {
                Some((freq, count)) => {
                    self.check_tracer(idx, src, tr.endpos, forward, right, bullet, (*freq, count))
                }
                None => false,
            };

            if tr.fraction != 1.0 {
                match bullet {
                    Bullet::PlayerMp5 => {
                        if !tracer {
                            self.play_texture_sound(idx, &tr, src, end, bullet);
                            self.decal_gunshot(&tr, bullet);
                        }
                    }
                    Bullet::PlayerBuckshot => {
                        self.decal_gunshot(&tr, bullet);
                    }
                    Bullet::Player357 => {
                        self.play_texture_sound(idx, &tr, src, end, bullet);
                        self.decal_gunshot(&tr, bullet);
                    }
                    _ => {
                        self.play_texture_sound(idx, &tr, src, end, bullet);
                        self.decal_gunshot(&tr, bullet);
                    }
                }
            }

            pm_states.pop();
        }
    }
}
