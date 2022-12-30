#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

use glam::{vec3, Vec3};
use once_cell::sync::Lazy;
use rand::Rng;
use wasm_bindgen::prelude::*;

use solver::hashing_11::Hash;

const TIME_STEP: f32 = 1.0 / 60.0;
const RADIUS: f32 = 0.025;
const MIN_DIST: f32 = 2.0 * RADIUS;
const MIN_DIST_SQ: f32 = MIN_DIST * MIN_DIST;
const SPACING: f32 = 3.0 * RADIUS;
const INIT_VEL_RAND: f32 = 0.2;
const BOUNDS: [Vec3; 2] = [vec3(-1.0, 0.0, -1.0), vec3(1.0, 2.0, 1.0)];

static NUMX: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].x - BOUNDS[0].x - 2.0 * SPACING) / SPACING) as usize);
static NUMY: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].y - BOUNDS[0].y - 2.0 * SPACING) / SPACING) as usize);
static NUMZ: Lazy<usize> =
    Lazy::new(|| f32::floor((BOUNDS[1].z - BOUNDS[0].z - 2.0 * SPACING) / SPACING) as usize);
static NUM_BODIES: Lazy<usize> = Lazy::new(|| *NUMX * *NUMY * *NUMZ);

#[wasm_bindgen]
pub struct HashSimulation {
    num_bodies: usize,
    pos: Vec<Vec3>,
    collisions: Vec<u8>, // store as u8 rather than bool so we can share directly with JS
    prev: Vec<Vec3>,
    vel: Vec<Vec3>,
    hash: Hash,
}

#[wasm_bindgen]
impl HashSimulation {
    #[allow(clippy::new_without_default)]
    #[wasm_bindgen(constructor)]
    pub fn new() -> HashSimulation {
        let mut sim = Self {
            num_bodies: *NUM_BODIES,
            pos: vec![Vec3::ZERO; *NUM_BODIES],
            collisions: vec![0; *NUM_BODIES],
            prev: vec![Vec3::ZERO; *NUM_BODIES],
            vel: vec![Vec3::ZERO; *NUM_BODIES],
            hash: Hash::new(MIN_DIST, *NUM_BODIES),
        };
        sim.reset();
        sim
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
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

    #[wasm_bindgen]
    pub fn step(&mut self) {
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

    // manually define since `#[wasm_bindgen]` doesn't yet work for constants
    #[wasm_bindgen]
    pub fn num_bodies() -> usize {
        *NUM_BODIES
    }

    #[wasm_bindgen]
    pub fn radius() -> f32 {
        RADIUS
    }

    #[wasm_bindgen]
    pub fn body_positions_ptr(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn body_collisions_ptr(&self) -> *const u8 {
        // See above comment for `body_positions_ptr` re: safety
        self.collisions.as_ptr()
    }
}
