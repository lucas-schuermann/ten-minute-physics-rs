use glam::{vec3, Vec3};
use once_cell::sync::Lazy;
use rand::Rng;

use solver::hash::Hash;

const TIME_STEP: f32 = 1.0 / 60.0;
pub(crate) const RADIUS: f32 = 0.025;
const MIN_DIST: f32 = 2.0 * RADIUS;
const MIN_DIST_SQ: f32 = MIN_DIST * MIN_DIST;
const SPACING: f32 = 3.0 * RADIUS;
const INIT_VEL_RAND: f32 = 0.2;
const BOUNDS: [Vec3; 2] = [vec3(-1.0, 0.0, -1.0), vec3(1.0, 2.0, 1.0)];

const NUMX: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].x - BOUNDS[0].x - 2.0 * SPACING) / SPACING) as usize);
const NUMY: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].y - BOUNDS[0].y - 2.0 * SPACING) / SPACING) as usize);
const NUMZ: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].z - BOUNDS[0].z - 2.0 * SPACING) / SPACING) as usize);
pub(crate) const NUM_BODIES: Lazy<usize> = Lazy::new(|| *NUMX * *NUMY * *NUMZ);

pub(crate) struct Demo {
    num_bodies: usize,
    pub(crate) pos: Vec<Vec3>,
    pub(crate) collisions: Vec<u8>, // store as u8 rather than bool so we can share directly with JS
    prev: Vec<Vec3>,
    vel: Vec<Vec3>,
    hash: Hash,
}

impl Demo {
    pub(crate) fn new() -> Self {
        Self {
            num_bodies: *NUM_BODIES,
            pos: vec![Vec3::ZERO; *NUM_BODIES],
            collisions: vec![0; *NUM_BODIES],
            prev: vec![Vec3::ZERO; *NUM_BODIES],
            vel: vec![Vec3::ZERO; *NUM_BODIES],
            hash: Hash::new(MIN_DIST, *NUM_BODIES),
        }
    }

    pub(crate) fn init(&mut self) {
        let mut rng = rand::thread_rng();
        for xi in 0..*NUMX {
            for yi in 0..*NUMY {
                for zi in 0..*NUMZ {
                    let x = (xi * *NUMY + yi) * *NUMZ + zi;
                    self.pos[x] =
                        BOUNDS[0] + SPACING + Vec3::new(xi as f32, yi as f32, zi as f32) * SPACING;
                    let r = Vec3::new(rng.gen(), rng.gen(), rng.gen());
                    self.vel[x] = -INIT_VEL_RAND + 2.0 * INIT_VEL_RAND * r;
                }
            }
        }
    }

    pub(crate) fn update(&mut self) {
        // integrate
        for i in 0..self.num_bodies {
            self.prev[i] = self.pos[i];
            self.pos[i] += self.vel[i] * TIME_STEP;
        }

        self.hash.create(&self.pos);

        // handle collisions
        for i in 0..self.num_bodies {
            self.collisions[i] = 0;

            // world
            for d in 0..3 {
                if self.pos[i][d] < BOUNDS[0][d] + RADIUS {
                    self.pos[i][d] = BOUNDS[0][d] + RADIUS;
                    self.vel[i][d] = -self.vel[i][d];
                    self.collisions[i] = 1;
                }
                if self.pos[i][d] > BOUNDS[1][d] - RADIUS {
                    self.pos[i][d] = BOUNDS[1][d] - RADIUS;
                    self.vel[i][d] = -self.vel[i][d];
                    self.collisions[i] = 1;
                }
            }

            // body to body
            self.hash.query(&self.pos[i], MIN_DIST);
            for q in 0..self.hash.query_size {
                let j = self.hash.query_ids[q];
                let mut normal = self.pos[i] - self.pos[j];
                let d2 = normal.length_squared();
                if d2 > 0.0 && d2 < MIN_DIST_SQ {
                    let d = d2.sqrt();
                    normal /= d;
                    let corr = (MIN_DIST - d) * 0.5;
                    self.pos[i] += normal * corr;
                    self.pos[j] += normal * -corr;
                    let vi = self.vel[i].dot(normal);
                    let vj = self.vel[j].dot(normal);
                    self.vel[i] += normal * (vj - vi);
                    self.vel[j] += normal * (vi - vj);
                    self.collisions[i] = 1;
                }
            }
        }
    }
}
