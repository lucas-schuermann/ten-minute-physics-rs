use std::cell::SyncUnsafeCell;

use glam::{vec3, Vec3};
use rayon::prelude::*;
use wasm_bindgen::prelude::*;

#[cfg(feature = "parallel")]
// must be exported to init rayon thread pool with web workers
pub use wasm_bindgen_rayon::init_thread_pool;

const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 60.0;
const JACOBI_SCALE: f32 = 0.2;
const CLOTH_SPACING: f32 = 0.01;
const CLOTH_THICKNESS: f32 = 0.001;
const FRICTION: f32 = 0.01;
const DEFAULT_CLOTH_POS_Y: f32 = 2.2;
const DEFAULT_OBSTACLE_POS: Vec3 = vec3(0.0, 1.5, 0.0);
const DEFAULT_OBSTACLE_RADIUS: f32 = 0.3;

#[derive(Copy, Clone)]
struct SolverPass {
    first_constraint: usize,
    size: usize,
    independent: bool,
}

impl SolverPass {
    #[must_use]
    fn new(first_constraint: usize, size: usize, independent: bool) -> Self {
        Self {
            first_constraint,
            size,
            independent,
        }
    }
}

#[wasm_bindgen]
#[derive(PartialEq, Copy, Clone)]
pub enum SolverKind {
    COLORING,
    JACOBI,
}

#[wasm_bindgen]
pub struct ParallelClothSimulation {
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    #[wasm_bindgen(readonly)]
    pub num_tris: usize,
    #[wasm_bindgen(readonly)]
    pub num_dist_constraints: usize,
    #[wasm_bindgen(readonly)]
    pub num_substeps: u8,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    inv_dt: f32,

    tri_ids: Vec<[usize; 3]>,
    passes: Vec<SolverPass>,
    dist_constraint_ids: Vec<(usize, usize)>,
    rest_lengths: Vec<f32>,
    pub solver_kind: SolverKind,

    pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    init_pos: Vec<Vec3>,
    inv_mass: Vec<f32>,
    corr: Vec<Vec3>,
    vel: Vec<Vec3>,

    obstacle_pos: Vec3,
    #[wasm_bindgen(readonly)]
    pub obstacle_radius: f32,

    grab_inv_mass: f32,
    grab_id: Option<usize>,
}

// mark as unsafe, as it's possible to provide parameters that cause
// undefined behaviour
unsafe fn add_unsync(vec_ptr: *mut &mut Vec<Vec3>, idx: usize, rhs: Vec3) {
    let vec = vec_ptr.read();
    let first_elem = vec.as_mut_ptr();
    *first_elem.add(idx) += rhs;
}

unsafe fn get_unsync(vec_ptr: *mut &mut Vec<Vec3>, idx: usize) -> Vec3 {
    let vec = vec_ptr.read();
    let first_elem = vec.as_mut_ptr();
    *first_elem.add(idx)
}

#[wasm_bindgen]
impl ParallelClothSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(num_substeps: u8, num_x: usize, num_y: usize) -> Self {
        let mut num_x = num_x;
        let mut num_y = num_y;
        if num_x % 2 == 1 {
            num_x = num_x + 1;
        }
        if num_y % 2 == 1 {
            num_y = num_y + 1;
        }

        let num_particles = (num_x + 1) * (num_y + 1);
        let mut pos = vec![Vec3::ZERO; num_particles];

        for xi in 0..num_x + 1 {
            for yi in 0..num_y + 1 {
                let id = xi * (num_y + 1) + yi;
                pos[id] = vec3(
                    (num_x as f32 * -0.5 + xi as f32) * CLOTH_SPACING,
                    DEFAULT_CLOTH_POS_Y,
                    (num_y as f32 * -0.5 + yi as f32) * CLOTH_SPACING,
                );
            }
        }

        let mut last_constraint_index = 0;
        let mut passes = vec![];
        for (size, independent) in [
            ((num_x + 1) * f32::floor(num_y as f32 / 2.0) as usize, true),
            ((num_x + 1) * f32::floor(num_y as f32 / 2.0) as usize, true),
            (f32::floor(num_x as f32 / 2.0) as usize * (num_y + 1), true),
            (f32::floor(num_x as f32 / 2.0) as usize * (num_y + 1), true),
            (
                2 * num_x * num_y + (num_x + 1) * (num_y - 1) + (num_y + 1) * (num_x - 1),
                false,
            ),
        ] {
            passes.push(SolverPass::new(last_constraint_index, size, independent));
            last_constraint_index += size;
        }
        let num_dist_constraints: usize = last_constraint_index;
        let mut dist_constraint_ids: Vec<(usize, usize)> = vec![(0, 0); num_dist_constraints];

        // stretch constraints
        let mut i = 0;
        for pass in 0..2 {
            for xi in 0..num_x + 1 {
                for yi in 0..f32::floor(num_y as f32 / 2.0) as usize {
                    dist_constraint_ids[i] = (
                        xi * (num_y + 1) + 2 * yi + pass,
                        xi * (num_y + 1) + 2 * yi + pass + 1,
                    );
                    i += 1;
                }
            }
        }
        for pass in 0..2 {
            for xi in 0..f32::floor(num_x as f32 / 2.0) as usize {
                for yi in 0..num_y + 1 {
                    dist_constraint_ids[i] = (
                        (2 * xi + pass) * (num_y + 1) + yi,
                        (2 * xi + pass + 1) * (num_y + 1) + yi,
                    );
                    i += 1;
                }
            }
        }

        // shear constraints
        for xi in 0..num_x {
            for yi in 0..num_y {
                dist_constraint_ids[i] = (xi * (num_y + 1) + yi, (xi + 1) * (num_y + 1) + yi + 1);
                i += 1;
                dist_constraint_ids[i] = ((xi + 1) * (num_y + 1) + yi, xi * (num_y + 1) + yi + 1);
                i += 1;
            }
        }

        // bending constraints
        for xi in 0..num_x + 1 {
            for yi in 0..num_y - 1 {
                dist_constraint_ids[i] = (xi * (num_y + 1) + yi, xi * (num_y + 1) + yi + 2);
                i += 1;
            }
        }
        for xi in 0..num_x - 1 {
            for yi in 0..num_y + 1 {
                dist_constraint_ids[i] = (xi * (num_y + 1) + yi, (xi + 2) * (num_y + 1) + yi);
                i += 1;
            }
        }

        // compute rest lengths
        let mut rest_lengths = vec![0.0; num_dist_constraints];
        for i in 0..num_dist_constraints {
            let (i0, i1) = dist_constraint_ids[i];
            let p0 = pos[i0];
            let p1 = pos[i1];
            rest_lengths[i] = (p1 - p0).length();
        }

        // compute tri ids
        let num_tris = 2 * num_x * num_y;
        let mut tri_ids = vec![[0; 3]; num_tris];
        let mut i = 0;
        for xi in 0..num_x {
            for yi in 0..num_y {
                let id0 = xi * (num_y + 1) + yi;
                let id1 = (xi + 1) * (num_y + 1) + yi;
                let id2 = (xi + 1) * (num_y + 1) + yi + 1;
                let id3 = xi * (num_y + 1) + yi + 1;
                tri_ids[i] = [id0, id1, id2];
                tri_ids[i + 1] = [id0, id2, id3];
                i += 2;
            }
        }

        let dt = TIME_STEP / Into::<f32>::into(num_substeps);
        Self {
            num_particles,
            num_tris,
            num_dist_constraints,
            num_substeps,
            dt,
            inv_dt: 1.0 / dt,

            tri_ids,
            passes,
            dist_constraint_ids,
            rest_lengths,
            solver_kind: SolverKind::JACOBI,

            pos: pos.clone(),
            prev: pos.clone(),
            init_pos: pos,
            inv_mass: vec![1.0; num_particles],
            corr: vec![Vec3::ZERO; num_particles],
            vel: vec![Vec3::ZERO; num_particles],

            obstacle_pos: DEFAULT_OBSTACLE_POS,
            obstacle_radius: DEFAULT_OBSTACLE_RADIUS,

            grab_inv_mass: 0.0,
            grab_id: None,
        }
    }

    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> *const Vec3 {
        self.pos.as_ptr()
    }

    #[wasm_bindgen(getter)]
    pub fn obstacle_pos(&self) -> Vec<f32> {
        self.obstacle_pos.to_array().to_vec()
    }

    pub fn get_pos_copy(&self) -> Vec<f32> {
        self.pos.iter().flat_map(|v| v.to_array()).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.tri_ids.iter().flat_map(|e| e.to_vec()).collect()
    }

    pub fn reset(&mut self) {
        self.pos.copy_from_slice(&self.init_pos);
        self.prev.copy_from_slice(&self.pos);
        self.vel.fill(Vec3::ZERO);
    }

    #[wasm_bindgen(setter)]
    pub fn set_num_substeps(&mut self, num_substeps: u8) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / Into::<f32>::into(num_substeps);
        self.inv_dt = 1.0 / self.dt;
    }

    fn integrate(&mut self) {
        (0..self.num_particles)
            .into_par_iter()
            .zip_eq(&mut self.pos)
            .zip_eq(&mut self.prev)
            .zip_eq(&mut self.vel)
            .for_each(|(((i, pos), prev), vel)| {
                *vel = (*pos - *prev) / self.dt;
                *prev = *pos;
                if self.inv_mass[i] == 0.0 {
                    return;
                }
                *vel += GRAVITY * self.dt;
                *pos += *vel * self.dt;

                // collisions
                let d = (*pos - self.obstacle_pos).length();
                if d < self.obstacle_radius + CLOTH_THICKNESS {
                    let p = *pos * (1.0 - FRICTION) + *prev * FRICTION;
                    let r = p - self.obstacle_pos;
                    let d = r.length();
                    *pos = self.obstacle_pos + r * ((self.obstacle_radius + CLOTH_THICKNESS) / d);
                }

                let mut p = *pos;
                if p.y < CLOTH_THICKNESS {
                    p = *pos * (1.0 - FRICTION) + *prev * FRICTION;
                    p.y = CLOTH_THICKNESS;
                    *pos = p;
                }
            });
    }

    fn solve_distance_constraints(
        &mut self,
        solver_kind: SolverKind,
        num_constraints: usize,
        first_constraint: usize,
    ) {
        let corr_cell = SyncUnsafeCell::new(&mut self.corr);
        let pos_cell = SyncUnsafeCell::new(&mut self.pos);

        (0..num_constraints).into_par_iter().for_each(|i| {
            let cid = first_constraint + i;
            let (id0, id1) = self.dist_constraint_ids[cid];
            let w0 = self.inv_mass[id0];
            let w1 = self.inv_mass[id1];
            let w = w0 + w1;
            if w == 0.0 {
                return;
            }
            let p0: Vec3;
            let p1: Vec3;
            unsafe {
                p0 = get_unsync(pos_cell.get(), id0);
                p1 = get_unsync(pos_cell.get(), id1);
            }
            let d = p1 - p0;
            let n = d.normalize();
            let l = d.length();
            let l0 = self.rest_lengths[cid];
            let dp = n * (l - l0) / w;
            // LVSTODO: comment on limitations
            if solver_kind == SolverKind::JACOBI {
                unsafe {
                    add_unsync(corr_cell.get(), id0, w0 * dp);
                    add_unsync(corr_cell.get(), id1, -w1 * dp);
                }
            } else {
                unsafe {
                    add_unsync(pos_cell.get(), id0, w0 * dp);
                    add_unsync(pos_cell.get(), id1, -w1 * dp);
                }
            }
        });
    }

    fn add_corrections(&mut self, scale: f32) {
        (0..self.num_particles)
            .into_par_iter()
            .zip_eq(&mut self.pos)
            .for_each(|(i, pos)| {
                *pos += self.corr[i] * scale;
            })
    }

    pub fn step(&mut self) {
        let passes = self.passes.clone(); // LVSTODO: can we get around clone?
        for _ in 0..self.num_substeps {
            self.integrate();
            match self.solver_kind {
                SolverKind::COLORING => {
                    for pass in &passes {
                        let num_constraints = pass.size;
                        if pass.independent {
                            self.solve_distance_constraints(
                                SolverKind::COLORING,
                                num_constraints,
                                pass.first_constraint,
                            );
                        } else {
                            self.corr.fill(Vec3::ZERO);
                            self.solve_distance_constraints(
                                SolverKind::JACOBI,
                                num_constraints,
                                pass.first_constraint,
                            );
                            self.add_corrections(JACOBI_SCALE);
                        }
                    }
                }
                SolverKind::JACOBI => {
                    self.corr.fill(Vec3::ZERO);
                    self.solve_distance_constraints(
                        SolverKind::JACOBI,
                        self.num_dist_constraints,
                        0,
                    );
                    self.add_corrections(JACOBI_SCALE);
                }
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

    pub fn end_grab(&mut self, _: usize, _: &[f32]) {
        if let Some(i) = self.grab_id {
            self.inv_mass[i] = self.grab_inv_mass;
        }
        self.grab_id = None;
    }
}
