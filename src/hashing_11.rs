use glam::{vec3, Vec3};
use once_cell::sync::Lazy;
use rand::Rng;
use wasm_bindgen::prelude::*;

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

pub struct Hash {
    inv_spacing: f32,
    table_size: usize,
    cell_start: Vec<usize>,
    cell_entries: Vec<usize>,
    pub query_ids: Vec<usize>,
    pub query_size: usize,

    // for `query_all`
    max_num_objects: usize,
    pub first_adj_id: Vec<usize>,
    pub adj_ids: Vec<usize>,
}

impl Hash {
    #[must_use]
    pub fn new(spacing: f32, max_num_objects: usize) -> Self {
        let table_size = 2 * max_num_objects;
        Self {
            inv_spacing: 1.0 / spacing,
            table_size,
            cell_start: vec![0; table_size + 1],
            cell_entries: vec![0; max_num_objects],
            query_ids: vec![0; max_num_objects],
            query_size: 0,

            // for `query_all`
            max_num_objects,
            first_adj_id: vec![0; max_num_objects + 1],
            adj_ids: Vec::with_capacity(10 * max_num_objects),
        }
    }

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_wrap)]
    fn hash_coords(&self, xi: i32, yi: i32, zi: i32) -> usize {
        (i32::abs(
            xi.wrapping_mul(92_837_111_i32)
                ^ yi.wrapping_mul(689_287_499_i32)
                ^ zi.wrapping_mul(283_923_481_i32),
        ) % (self.table_size as i32)) as usize // fantasy function
    }

    fn int_coord(&self, coord: f32) -> i32 {
        f32::floor(coord * self.inv_spacing) as i32
    }

    fn hash_pos(&self, pos: &Vec3) -> usize {
        self.hash_coords(
            self.int_coord(pos.x),
            self.int_coord(pos.y),
            self.int_coord(pos.z),
        )
    }

    pub fn create(&mut self, positions: &[Vec3]) {
        // compute cell sizes
        self.cell_start.fill(0);
        self.cell_entries.fill(0);
        for &p in positions {
            let h = self.hash_pos(&p);
            self.cell_start[h] += 1;
        }

        // compute cell starts
        let mut start = 0;
        for i in 0..self.table_size {
            start += self.cell_start[i];
            self.cell_start[i] = start;
        }
        self.cell_start[self.table_size] = start; // guard

        // fill in object ids
        for (i, pos) in positions.iter().enumerate().take(self.cell_entries.len()) {
            let h = self.hash_pos(pos);
            self.cell_start[h] -= 1;
            self.cell_entries[self.cell_start[h]] = i;
        }
    }

    pub fn query(&mut self, pos: &Vec3, max_dist: f32) {
        let x0 = self.int_coord(pos.x - max_dist);
        let y0 = self.int_coord(pos.y - max_dist);
        let z0 = self.int_coord(pos.z - max_dist);

        let x1 = self.int_coord(pos.x + max_dist);
        let y1 = self.int_coord(pos.y + max_dist);
        let z1 = self.int_coord(pos.z + max_dist);

        self.query_size = 0;

        for xi in x0..=x1 {
            for yi in y0..=y1 {
                for zi in z0..=z1 {
                    let h = self.hash_coords(xi, yi, zi);
                    let start = self.cell_start[h];
                    let end = self.cell_start[h + 1];

                    for i in start..end {
                        self.query_ids[self.query_size] = self.cell_entries[i];
                        self.query_size += 1;
                    }
                }
            }
        }
    }

    // for use in `self_collision_15.rs`
    pub fn query_all(&mut self, positions: &[Vec3], max_dist: f32) {
        let max_dist_sq = max_dist * max_dist;
        self.adj_ids.clear();
        for i in 0..self.max_num_objects {
            let id0 = i;
            self.first_adj_id[id0] = self.adj_ids.len();
            self.query(&positions[id0], max_dist);

            for j in 0..self.query_size {
                let id1 = self.query_ids[j];
                if id1 >= id0 {
                    continue;
                }
                let dist_sq = positions[id0].distance_squared(positions[id1]);
                if dist_sq > max_dist_sq {
                    continue;
                }
                self.adj_ids.push(id1);
            }
        }
        self.first_adj_id[self.max_num_objects] = self.adj_ids.len();
    }
}

#[wasm_bindgen]
pub struct HashSimulation {
    #[wasm_bindgen(readonly)]
    pub num_bodies: usize,
    pos: Vec<Vec3>,
    collisions: Vec<u8>, // store as u8 rather than bool so we can share directly with JS
    prev: Vec<Vec3>,
    vel: Vec<Vec3>,
    hash: Hash,
}

#[wasm_bindgen]
impl HashSimulation {
    #[allow(clippy::new_without_default)]
    #[must_use]
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
    #[wasm_bindgen(getter)]
    pub fn radius() -> f32 {
        RADIUS
    }

    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.pos.as_ptr()
    }

    #[wasm_bindgen(getter)]
    pub fn collisions(&self) -> *const u8 {
        // See above comment for `pos` re: safety
        self.collisions.as_ptr()
    }
}
