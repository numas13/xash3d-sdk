use xash3d_shared::{consts::PM_STUDIO_BOX, ffi::common::vec3_t, math::ToAngleVectors};

const BOX_GAP: f32 = 0.0;
const BOX_POINTS: [[usize; 4]; 6] = [
    [0, 4, 6, 2], // +X
    [0, 1, 5, 4], // +Y
    [0, 2, 3, 1], // +Z
    [7, 5, 1, 3], // -X
    [7, 3, 2, 6], // -Y
    [7, 6, 4, 5], // -Z
];

const FRAMERATE: u32 = 60;
const PARTICLE_LIFETIME: f32 = 1.0 / (FRAMERATE as f32) * 1.5;

impl super::PlayerMove<'_> {
    fn draw_particle_line(&self, start: vec3_t, end: vec3_t, color: i32) {
        let linestep = 2.0;
        let (diff, len) = (end - start).normalize_length();
        let mut curdist = 0.0;
        while curdist <= len {
            let curpos = start + diff * curdist;
            self.particle(curpos, color, PARTICLE_LIFETIME, 0, 0);
            curdist += linestep;
        }
    }

    fn draw_rectangle(&self, tl: vec3_t, bl: vec3_t, tr: vec3_t, br: vec3_t, color: i32) {
        self.draw_particle_line(tl, bl, color);
        self.draw_particle_line(bl, br, color);
        self.draw_particle_line(br, tr, color);
        self.draw_particle_line(tr, tl, color);
    }

    fn draw_phys_ent_bbox(&self, num: i32, color: i32) {
        let mut points = [vec3_t::ZERO; 8];
        let gap = BOX_GAP;

        if num >= self.raw.numphysent || num <= 0 {
            return;
        }

        let pe = &self.raw.physents[num as usize];

        if !pe.model.is_null() {
            let model = unsafe { &*pe.model };
            let (mins, maxs) = self.get_model_bounds(model);

            for (i, point) in points.iter_mut().enumerate() {
                let x = if i & 1 != 0 {
                    mins[0] - gap
                } else {
                    maxs[0] + gap
                };
                let y = if i & 2 != 0 {
                    mins[1] - gap
                } else {
                    maxs[1] + gap
                };
                let z = if i & 4 != 0 {
                    mins[2] - gap
                } else {
                    maxs[2] + gap
                };
                *point = pe.origin + vec3_t::new(x, y, z);
            }

            if pe.angles != vec3_t::ZERO {
                let av = pe.angles.angle_vectors().transpose_all();
                for point in &mut points {
                    *point = vec3_t::new(
                        point.dot_product(av.forward),
                        point.dot_product(av.right),
                        point.dot_product(av.up),
                    );
                }
            }

            for point in &mut points {
                *point += pe.origin;
            }

            for i in 0..6 {
                self.draw_rectangle(
                    points[BOX_POINTS[i][1]],
                    points[BOX_POINTS[i][0]],
                    points[BOX_POINTS[i][2]],
                    points[BOX_POINTS[i][3]],
                    color,
                );
            }
        } else {
            for (i, point) in points.iter_mut().enumerate() {
                let x = if i & 1 != 0 { pe.mins[0] } else { pe.maxs[0] };
                let y = if i & 2 != 0 { pe.mins[1] } else { pe.maxs[1] };
                let z = if i & 4 != 0 { pe.mins[2] } else { pe.maxs[2] };
                *point = pe.origin + vec3_t::new(x, y, z);
            }

            for i in 0..6 {
                self.draw_rectangle(
                    points[BOX_POINTS[i][1]],
                    points[BOX_POINTS[i][0]],
                    points[BOX_POINTS[i][2]],
                    points[BOX_POINTS[i][3]],
                    color,
                );
            }
        }
    }

    fn view_entity(&self, color: i32) {
        let raydist = 256.0;
        let forward = self.raw.angles.angle_vectors().forward();
        let start = self.raw.origin;
        let start = start.copy_with_z(self.raw.origin[2] + self.raw.view_ofs[2]);
        let end = start + forward * raydist;
        let trace = self.player_trace(start, end, PM_STUDIO_BOX, -1);
        if trace.ent > 0 {
            self.draw_phys_ent_bbox(trace.ent, color);
        }
    }

    fn draw_bbox(&self, mins: vec3_t, maxs: vec3_t, origin: vec3_t, color: i32) {
        let mut points = [vec3_t::ZERO; 8];
        let gap = BOX_GAP;

        for (i, point) in points.iter_mut().enumerate() {
            let x = if i & 1 != 0 {
                mins[0] - gap
            } else {
                maxs[0] + gap
            };
            let y = if i & 2 != 0 {
                mins[1] - gap
            } else {
                maxs[1] + gap
            };
            let z = if i & 4 != 0 {
                mins[2] - gap
            } else {
                maxs[2] + gap
            };
            *point = origin + vec3_t::new(x, y, z);
        }

        for i in 0..6 {
            self.draw_rectangle(
                points[BOX_POINTS[i][1]],
                points[BOX_POINTS[i][0]],
                points[BOX_POINTS[i][2]],
                points[BOX_POINTS[i][3]],
                color,
            );
        }
    }

    pub(super) fn show_clip_box(&self) {
        if self.raw.runfuncs == 0 {
            return;
        }

        if true {
            self.view_entity(111);
        }

        let color = 132;
        if true {
            self.draw_bbox(
                self.raw.player_mins[self.usehull()],
                self.raw.player_maxs[self.usehull()],
                self.raw.origin,
                color,
            );
        }

        if true {
            let raydist = 256.0;
            let forward = self.raw.angles.angle_vectors().forward();
            let start = self.raw.origin;
            let end = start + forward * raydist;
            self.draw_particle_line(start, end, color);
        }
    }
}
