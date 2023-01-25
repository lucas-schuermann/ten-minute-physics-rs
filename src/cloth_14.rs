use std::cmp::Ordering;

use glam::{vec3, Vec3};
use rand::Rng;
use wasm_bindgen::prelude::*;

use crate::mesh::{self, MeshData};

const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 60.0;
const INITIAL_VEL_SCALING: f32 = 0.0001;
const ATTACHMENT_EPSILON: f32 = 0.0001;

#[wasm_bindgen]
pub struct ClothSimulation {
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    #[wasm_bindgen(readonly)]
    pub num_tris: usize,
    #[wasm_bindgen(readonly)]
    pub num_substeps: u8,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    inv_dt: f32,

    edge_ids: Vec<[usize; 2]>,
    tri_ids: Vec<[usize; 3]>,

    pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    vel: Vec<Vec3>,
    inv_mass: Vec<f32>,

    stretching_ids: Vec<[usize; 2]>,
    bending_ids: Vec<[usize; 4]>,
    stretching_lengths: Vec<f32>,
    bending_lengths: Vec<f32>,

    grab_inv_mass: f32,
    grab_id: Option<usize>,

    pub bending_compliance: f32,
    pub stretching_compliance: f32,

    // stored for reset
    mesh: MeshData,
}

struct Edge {
    id0: usize,
    id1: usize,
    edge_num: usize,
}

fn find_tri_neighbors(tri_ids: &Vec<[usize; 3]>) -> Vec<Option<usize>> {
    // create common edges
    let num_edges = tri_ids.len() * 3;
    let mut edges = Vec::with_capacity(num_edges);
    for (i, ids) in tri_ids.iter().enumerate() {
        for j in 0..3 {
            let id0 = ids[j];
            let id1 = ids[(j + 1) % 3];
            edges.push(Edge {
                id0: id0.min(id1),
                id1: id0.max(id1),
                edge_num: 3 * i + j,
            });
        }
    }

    // soft so common edges are next to each other
    edges.sort_by(|a, b| {
        if (a.id0 < b.id0) || (a.id0 == b.id0 && a.id1 < b.id1) {
            Ordering::Less
        } else {
            Ordering::Greater
        }
    });

    // find matching edges
    let mut neighbors: Vec<Option<usize>> = vec![None; num_edges];

    let mut i = 0;
    while i < edges.len() {
        let e0 = &edges[i];
        i += 1;
        if i < edges.len() {
            let e1 = &edges[i];
            if e0.id0 == e1.id0 && e0.id1 == e1.id1 {
                neighbors[e0.edge_num] = Some(e1.edge_num);
                neighbors[e1.edge_num] = Some(e0.edge_num);
            }
            i += 1;
        }
    }

    neighbors
}

#[wasm_bindgen]
impl ClothSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(num_substeps: u8, bending_compliance: f32, stretching_compliance: f32) -> Self {
        let mesh = mesh::get_cloth();
        let num_particles = mesh.vertices.len();
        let num_tris = mesh.tri_ids.len();
        let pos = mesh.vertices.clone();

        let neighbors = find_tri_neighbors(&mesh.tri_ids);

        let mut edge_ids = vec![];
        let mut tri_pair_ids = vec![];

        for i in 0..num_tris {
            for j in 0..3 {
                let id0 = mesh.tri_ids[i][j];
                let id1 = mesh.tri_ids[i][(j + 1) % 3];

                // each edge only once
                let n = neighbors[3 * i + j];
                if n.is_none() || id0 < id1 {
                    edge_ids.push([id0, id1]);
                }
                // tri pair
                if let Some(n) = n {
                    let ni = f32::floor(n as f32 / 3.0) as usize;
                    let nj = n % 3;
                    let id2 = mesh.tri_ids[i][(j + 2) % 3];
                    let id3 = mesh.tri_ids[ni][(nj + 2) % 3];
                    tri_pair_ids.push([id0, id1, id2, id3]);
                }
            }
        }

        let dt = TIME_STEP / Into::<f32>::into(num_substeps);
        let mut cloth = Self {
            num_particles,
            num_tris,
            num_substeps,
            dt,
            inv_dt: 1.0 / dt,

            edge_ids: edge_ids.clone(),
            tri_ids: mesh.tri_ids.clone(),

            pos,
            prev: mesh.vertices.clone(),
            vel: vec![Vec3::ZERO; num_particles],
            inv_mass: vec![0.0; num_particles],

            stretching_ids: edge_ids.clone(),
            bending_ids: tri_pair_ids.clone(),
            stretching_lengths: vec![0.0; edge_ids.len()],
            bending_lengths: vec![0.0; tri_pair_ids.len()],

            grab_inv_mass: 0.0,
            grab_id: None,

            bending_compliance,
            stretching_compliance,

            mesh,
        };
        cloth.init();
        cloth
    }

    fn randomize_vels(&mut self) {
        // slightly perturb initial velocities for better drop visual
        let mut rng = rand::thread_rng();
        self.vel
            .iter_mut()
            .for_each(|v| *v = Vec3::splat(rng.gen::<f32>() * INITIAL_VEL_SCALING));
    }

    #[wasm_bindgen(getter)]
    pub fn pos(&self) -> *const Vec3 {
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

    pub fn reset(&mut self) {
        self.pos.copy_from_slice(&self.mesh.vertices);
        self.prev.copy_from_slice(&self.pos);
        self.randomize_vels();
    }

    #[wasm_bindgen(setter)]
    pub fn set_solver_substeps(&mut self, num_substeps: u8) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / Into::<f32>::into(num_substeps);
        self.inv_dt = 1.0 / self.dt;
    }

    fn init(&mut self) {
        self.inv_mass = vec![0.0; self.num_particles];
        let mut e0;
        let mut e1;
        let mut c;

        for id in &self.tri_ids {
            let (id0, id1, id2) = (id[0], id[1], id[2]);
            e0 = self.pos[id1] - self.pos[id0];
            e1 = self.pos[id2] - self.pos[id0];
            c = e0.cross(e1);
            let a = 0.5 * c.length();
            let p_inv_mass = if a > 0.0 { 1.0 / a / 3.0 } else { 0.0 };
            self.inv_mass[id0] += p_inv_mass;
            self.inv_mass[id1] += p_inv_mass;
            self.inv_mass[id2] += p_inv_mass;
        }

        for i in 0..self.stretching_lengths.len() {
            let id = self.stretching_ids[i];
            self.stretching_lengths[i] = (self.pos[id[0]] - self.pos[id[1]]).length();
        }

        for i in 0..self.bending_lengths.len() {
            let id = self.bending_ids[i];
            self.bending_lengths[i] = (self.pos[id[2]] - self.pos[id[3]]).length();
        }

        // attach
        let mut min_x = f32::MAX;
        let mut max_x = -f32::MAX;
        let mut max_y = -f32::MAX;

        for p in &self.pos {
            min_x = min_x.min(p.x);
            max_x = max_x.max(p.x);
            max_y = max_y.max(p.y);
        }

        for (i, p) in self.pos.iter().enumerate() {
            if (p.y > max_y - ATTACHMENT_EPSILON)
                && (p.x < min_x + ATTACHMENT_EPSILON || p.x > max_x - ATTACHMENT_EPSILON)
            {
                self.inv_mass[i] = 0.0;
            }
        }

        self.randomize_vels();
    }

    fn pre_solve(&mut self) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            self.vel[i] += GRAVITY * self.dt;
            self.prev[i] = self.pos[i];
            self.pos[i] += self.vel[i] * self.dt;

            // boundary condition (floor)
            if self.pos[i].y < 0.0 {
                self.pos[i] = self.prev[i];
                self.pos[i].y = 0.0;
            }
        }
    }

    fn solve(&mut self) {
        self.solve_stretching();
        self.solve_bending();
    }

    fn post_solve(&mut self) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            self.vel[i] = (self.pos[i] - self.prev[i]) * self.inv_dt;
        }
    }

    fn solve_stretching(&mut self) {
        let alpha = self.stretching_compliance * self.inv_dt * self.inv_dt;
        for i in 0..self.stretching_lengths.len() {
            let id = self.stretching_ids[i];
            let (id0, id1) = (id[0], id[1]);
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
            let rest_len = self.stretching_lengths[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            self.pos[id0] += grad * s * w0;
            self.pos[id1] += grad * -s * w1;
        }
    }

    fn solve_bending(&mut self) {
        let alpha = self.bending_compliance * self.inv_dt * self.inv_dt;
        for i in 0..self.bending_lengths.len() {
            let id = self.bending_ids[i];
            let (id0, id1) = (id[2], id[3]);
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
            let rest_len = self.bending_lengths[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            self.pos[id0] += grad * s * w0;
            self.pos[id1] += grad * -s * w1;
        }
    }

    pub fn step(&mut self) {
        for _ in 0..self.num_substeps {
            self.pre_solve();
            self.solve();
            self.post_solve();
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
