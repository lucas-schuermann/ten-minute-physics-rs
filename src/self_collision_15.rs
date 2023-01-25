use glam::{vec3, Vec3};
use rand::Rng;
use wasm_bindgen::prelude::*;

use crate::hashing_11::Hash;

const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 60.0;
const VEL_LIMIT_MULTIPLIER: f32 = 0.2;
const DEFAULT_THICKNESS: f32 = 0.01;
const SPACING: f32 = 0.01;
const DAMPING: f32 = 1.0;
const JITTER: f32 = -2.0 * (0.001 * SPACING) * (0.001 * SPACING);
const NUM_X: usize = 30;
const NUM_Y: usize = 200;
const CONSTRAINTS: [(ConstraintKind, (usize, usize, usize, usize)); 6] = [
    (ConstraintKind::Stretch, (0, 0, 0, 1)),
    (ConstraintKind::Stretch, (0, 0, 1, 0)),
    (ConstraintKind::Shear, (0, 0, 1, 1)),
    (ConstraintKind::Shear, (0, 1, 1, 0)),
    (ConstraintKind::Bending, (0, 0, 0, 2)),
    (ConstraintKind::Bending, (0, 0, 2, 0)),
];
const MAX_ADJ_IDS_MULTIPLIER: usize = 25;

#[derive(Default, Clone, Copy)]
enum ConstraintKind {
    Stretch,
    Shear,
    #[default]
    Bending,
}

#[derive(Default, Clone, Copy)]
struct Constraint {
    ids: (usize, usize),
    kind: ConstraintKind,
    rest_len: f32,
}

pub struct AdjHash {
    hash: Hash,

    max_num_objects: usize,
    pub first_adj_id: Vec<usize>,
    pub adj_ids: Vec<usize>,
}

impl AdjHash {
    #[must_use]
    pub fn new(spacing: f32, max_num_objects: usize) -> Self {
        Self {
            hash: Hash::new(spacing, max_num_objects),

            max_num_objects,
            first_adj_id: vec![0; max_num_objects + 1],
            adj_ids: Vec::with_capacity(MAX_ADJ_IDS_MULTIPLIER * max_num_objects),
        }
    }

    pub fn create(&mut self, positions: &[Vec3]) {
        self.hash.create(positions);
    }

    pub fn query_all(&mut self, positions: &[Vec3], max_dist: f32) {
        let max_dist_sq = max_dist * max_dist;
        self.adj_ids.clear();
        for i in 0..self.max_num_objects {
            let id0 = i;
            self.first_adj_id[id0] = self.adj_ids.len();
            self.hash.query(&positions[id0], max_dist);

            for j in 0..self.hash.query_size {
                let id1 = self.hash.query_ids[j];
                if id1 >= id0 {
                    continue;
                }
                let dist_sq = positions[id0].distance_squared(positions[id1]);
                if dist_sq > max_dist_sq {
                    continue;
                }
                if self.adj_ids.len() < self.adj_ids.capacity() {
                    self.adj_ids.push(id1);
                }
            }
        }
        self.first_adj_id[self.max_num_objects] = self.adj_ids.len();
    }
}

#[wasm_bindgen]
pub struct SelfCollisionSimulation {
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    #[wasm_bindgen(readonly)]
    pub num_tris: usize,
    num_substeps: u8,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    inv_dt: f32,
    max_vel: f32,

    edge_ids: Vec<[usize; 2]>,
    tri_ids: Vec<[usize; 3]>,

    pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    rest_pos: Vec<Vec3>,
    vel: Vec<Vec3>,
    inv_mass: Vec<f32>,
    thickness: f32,
    pub handle_collisions: bool,
    hash: AdjHash,

    grab_inv_mass: f32,
    grab_id: Option<usize>,

    num_constraints: usize,
    constraints: Vec<Constraint>,
    pub stretch_compliance: f32,
    pub shear_compliance: f32,
    pub bending_compliance: f32,
    pub friction: f32,
}

#[wasm_bindgen]
impl SelfCollisionSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(
        num_substeps: u8,
        bending_compliance: f32,
        stretch_compliance: f32,
        shear_compliance: f32,
        friction: f32,
    ) -> Self {
        let num_particles = NUM_X * NUM_Y;

        let mut edge_ids = vec![];
        let mut tri_ids = vec![];
        for i in 0..NUM_X {
            for j in 0..NUM_Y {
                let id = i * NUM_Y + j;
                if i < NUM_X - 1 && j < NUM_Y - 1 {
                    tri_ids.push([id + 1, id, id + 1 + NUM_Y]);
                    tri_ids.push([id + 1 + NUM_Y, id, id + NUM_Y]);
                }
                if i < NUM_X - 1 {
                    edge_ids.push([id, id + NUM_Y]);
                }
                if j < NUM_Y - 1 {
                    edge_ids.push([id, id + 1]);
                }
            }
        }

        let dt = TIME_STEP / Into::<f32>::into(num_substeps);
        let mut cloth = Self {
            num_particles,
            num_tris: tri_ids.len(),
            num_substeps,
            dt,
            inv_dt: 1.0 / dt,
            max_vel: VEL_LIMIT_MULTIPLIER * DEFAULT_THICKNESS / dt,

            edge_ids,
            tri_ids,

            pos: vec![Vec3::ZERO; num_particles],
            prev: vec![Vec3::ZERO; num_particles],
            rest_pos: vec![Vec3::ZERO; num_particles],
            vel: vec![Vec3::ZERO; num_particles],
            inv_mass: vec![0.0; num_particles],
            thickness: DEFAULT_THICKNESS,
            handle_collisions: true,
            hash: AdjHash::new(SPACING, num_particles),

            grab_inv_mass: 0.0,
            grab_id: None,

            num_constraints: 0,
            constraints: vec![Constraint::default(); num_particles * CONSTRAINTS.len()],
            stretch_compliance,
            shear_compliance,
            bending_compliance,
            friction,
        };
        cloth.init();
        cloth
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

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen(getter)]
    pub fn edge_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.edge_ids.iter().flat_map(|e| e.to_vec()).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.tri_ids.iter().flat_map(|e| e.to_vec()).collect()
    }

    pub fn reset(&mut self, attach: bool) {
        let mut rng = rand::thread_rng();
        for i in 0..NUM_X {
            for j in 0..NUM_Y {
                let id = i * NUM_Y + j;
                self.pos[id] = vec3(
                    -1.0 * NUM_X as f32 * SPACING * 0.5 + i as f32 * SPACING,
                    0.2 + j as f32 * SPACING,
                    0.0,
                );
                self.inv_mass[id] = 1.0;
                if attach && j == NUM_Y - 1 && (i == 0 || i == NUM_X - 1) {
                    self.inv_mass[id] = 0.0;
                }
            }
        }

        self.pos.iter_mut().for_each(|p| {
            p.x += JITTER * rng.gen::<f32>();
            p.y += JITTER * rng.gen::<f32>();
            p.z += JITTER * rng.gen::<f32>();
        });

        self.rest_pos.copy_from_slice(&self.pos);
        self.vel.fill(Vec3::ZERO);
    }

    #[wasm_bindgen(setter)]
    pub fn set_solver_substeps(&mut self, num_substeps: u8) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / Into::<f32>::into(num_substeps);
        self.inv_dt = 1.0 / self.dt;
        self.max_vel = VEL_LIMIT_MULTIPLIER * self.thickness / self.dt;
    }

    fn init(&mut self) {
        self.reset(false);

        self.num_constraints = 0;
        for (kind, indices) in CONSTRAINTS {
            for i in 0..NUM_X {
                for j in 0..NUM_Y {
                    let i0 = i + indices.0;
                    let j0 = j + indices.1;
                    let i1 = i + indices.2;
                    let j1 = j + indices.3;
                    if i0 < NUM_X && j0 < NUM_Y && i1 < NUM_X && j1 < NUM_Y {
                        let id0 = i0 * NUM_Y + j0;
                        let id1 = i1 * NUM_Y + j1;
                        let rest_len = self.pos[id0].distance(self.pos[id1]);
                        self.constraints[self.num_constraints] = Constraint {
                            ids: (id0, id1),
                            kind,
                            rest_len,
                        };
                        self.num_constraints += 1;
                    }
                }
            }
        }
    }

    #[must_use]
    fn get_compliance(&self, constraint: &Constraint) -> f32 {
        match constraint.kind {
            ConstraintKind::Stretch => self.stretch_compliance,
            ConstraintKind::Shear => self.shear_compliance,
            ConstraintKind::Bending => self.bending_compliance,
        }
    }

    pub fn step(&mut self) {
        if self.handle_collisions {
            self.hash.create(&self.pos);
            let max_dist = self.max_vel * self.dt * Into::<f32>::into(self.num_substeps);
            self.hash.query_all(&self.pos, max_dist);
        }

        for _ in 0..self.num_substeps {
            // integrate
            for i in 0..self.num_particles {
                if self.inv_mass[i] == 0.0 {
                    continue;
                }
                self.vel[i] += GRAVITY * self.dt;
                let v = self.vel[i].length();
                if v > self.max_vel {
                    self.vel[i] *= self.max_vel / v;
                }
                self.prev[i] = self.pos[i];
                self.pos[i] += self.vel[i] * self.dt;
            }

            // solve
            self.solve_ground_collisions();
            self.solve_constraints();
            if self.handle_collisions {
                self.solve_collisions();
            }

            // update velocities
            for i in 0..self.num_particles {
                if self.inv_mass[i] == 0.0 {
                    continue;
                }
                self.vel[i] = (self.pos[i] - self.prev[i]) * self.inv_dt;
            }
        }
    }

    fn solve_constraints(&mut self) {
        for cons in &self.constraints {
            let id0 = cons.ids.0;
            let id1 = cons.ids.1;
            let w0 = self.inv_mass[id0];
            let w1 = self.inv_mass[id1];
            let w = w0 + w1;
            if w == 0.0 {
                continue;
            }

            let mut grad = self.pos[id0] - self.pos[id1];
            let len = grad.length();
            if len == 0.0 {
                continue;
            }
            grad /= len;
            let c = len - cons.rest_len;
            let alpha = self.get_compliance(cons) * self.inv_dt * self.inv_dt;
            let s = -c / (w + alpha);
            self.pos[id0] += grad * s * w0;
            self.pos[id1] += grad * -s * w1;
        }
    }

    fn solve_ground_collisions(&mut self) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            if self.pos[i].y < 0.5 * self.thickness {
                let grad = self.pos[i] - self.prev[i];
                self.pos[i] += grad * -DAMPING;
                self.pos[i].y = 0.5 * self.thickness;
            }
        }
    }

    fn solve_collisions(&mut self) {
        let thickness_sq = self.thickness * self.thickness;
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            let id0 = i;
            let first = self.hash.first_adj_id[i];
            let last = self.hash.first_adj_id[i + 1];
            for j in first..last {
                let id1 = self.hash.adj_ids[j];
                if self.inv_mass[id1] == 0.0 {
                    continue;
                }
                let mut grad = self.pos[id1] - self.pos[id0];
                let dist_sq = grad.length_squared();
                if dist_sq > thickness_sq || dist_sq == 0.0 {
                    continue;
                }
                let rest_dist_sq = (self.rest_pos[id0] - self.rest_pos[id1]).length();
                let mut min_dist = self.thickness;
                if dist_sq > rest_dist_sq {
                    continue;
                }
                if rest_dist_sq < thickness_sq {
                    min_dist = rest_dist_sq.sqrt();
                }

                // position correction
                let dist = dist_sq.sqrt();
                grad *= (min_dist - dist) / dist;
                self.pos[id0] += grad * -0.5;
                self.pos[id1] += grad * 0.5;

                // friction
                let mut v0 = self.pos[id0] - self.prev[id0];
                let mut v1 = self.pos[id1] - self.prev[id1];
                let v_avg = (v0 + v1) / 2.0;
                // velocity correction
                v0 = v_avg - v0;
                v1 = v_avg - v1;
                // add corrections
                self.pos[id0] += v0 * self.friction;
                self.pos[id1] += v1 * self.friction;
            }
        }
    }

    pub fn start_grab(&mut self, _: usize, pos: &[f32]) {
        let pos = Vec3::from_slice(pos);
        let mut min_d2 = f32::MAX;
        self.grab_id = None;
        for i in 0..self.num_particles {
            let d2 = (pos - self.pos[i]).length_squared();
            if d2 < min_d2 {
                min_d2 = d2;
                self.grab_id = Some(i);
            }
        }

        if let Some(i) = self.grab_id {
            self.grab_inv_mass = self.inv_mass[i];
            self.inv_mass[i] = 0.0;
            self.pos[i] = pos;
        }
    }

    pub fn move_grabbed(&mut self, _: usize, pos: &[f32]) {
        let pos = Vec3::from_slice(pos);
        if let Some(i) = self.grab_id {
            self.pos[i] = pos;
        }
    }

    pub fn end_grab(&mut self, _: usize, vel: &[f32]) {
        let vel = Vec3::from_slice(vel);
        if let Some(i) = self.grab_id {
            self.inv_mass[i] = self.grab_inv_mass;
            self.vel[i] = vel;
        }
        self.grab_id = None;
    }
}
