use glam::{vec3, Mat3, Vec3};
use wasm_bindgen::prelude::*;

use crate::{
    hashing_11::Hash,
    mesh::{self, SkinnedTetMeshData},
};

const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 60.0;
const SPACING: f32 = 0.05;
const VOL_ID_ORDER: [[usize; 3]; 4] = [[1, 3, 2], [0, 2, 3], [0, 3, 1], [0, 1, 2]];
const SQUASH_TO_Y: f32 = 0.5;

#[wasm_bindgen]
pub struct SkinnedSoftbodySimulation {
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    #[wasm_bindgen(readonly)]
    pub num_tris: usize,
    #[wasm_bindgen(readonly)]
    pub num_tets: usize,
    #[wasm_bindgen(readonly)]
    pub num_surface_verts: usize,
    num_substeps: u8,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    inv_dt: f32,

    tet_ids: Vec<[usize; 4]>,
    edge_ids: Vec<usize>,
    skinning_info: Vec<Option<(usize, [f32; 3])>>,

    pos: Vec<Vec3>,
    surface_pos: Vec<Vec3>,
    prev: Vec<Vec3>,
    vel: Vec<Vec3>,
    inv_mass: Vec<f32>,
    rest_vol: Vec<f32>,
    edge_lens: Vec<f32>,

    grab_inv_mass: f32,
    grab_id: Option<usize>,

    pub edge_compliance: f32,
    pub vol_compliance: f32,

    // stored for reset
    mesh: SkinnedTetMeshData,
}

#[wasm_bindgen]
impl SkinnedSoftbodySimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(num_substeps: u8, edge_compliance: f32, vol_compliance: f32) -> Self {
        let mesh = mesh::get_dragon();
        let num_particles = mesh.tet_vertices.len();
        let num_tets = mesh.tet_ids.len();
        let num_surface_verts = mesh.surface_vertices.len();
        let dt = TIME_STEP / Into::<f32>::into(num_substeps);
        let mut body = Self {
            num_particles,
            num_tris: mesh.surface_tri_ids.len() / 3,
            num_tets,
            num_surface_verts,
            num_substeps,
            dt,
            inv_dt: 1.0 / dt,

            edge_ids: mesh.tet_edge_ids.clone(),
            tet_ids: mesh.tet_ids.clone(),
            skinning_info: vec![None; num_surface_verts],

            pos: mesh.tet_vertices.clone(),
            surface_pos: mesh.surface_vertices.clone(),
            prev: mesh.tet_vertices.clone(),
            vel: vec![Vec3::ZERO; num_particles],
            inv_mass: vec![0.0; num_particles],
            rest_vol: vec![0.0; num_tets],
            edge_lens: vec![0.0; mesh.tet_edge_ids.len() / 2],

            grab_inv_mass: 0.0,
            grab_id: None,

            edge_compliance,
            vol_compliance,

            mesh,
        };
        body.init();
        body
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
    pub fn surface_pos(&self) -> *const Vec3 {
        // See above comment for `pos` re: safety
        self.surface_pos.as_ptr()
    }

    // We can copy since we are not performance sensitive for these three methods
    #[wasm_bindgen(getter)]
    pub fn tet_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.tet_ids.iter().flat_map(|e| e.to_vec()).collect()
    }

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen(getter)]
    pub fn edge_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.edge_ids.clone()
    }

    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn surface_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.mesh.surface_tri_ids.clone()
    }

    #[wasm_bindgen(setter)]
    pub fn set_solver_substeps(&mut self, num_substeps: u8) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / Into::<f32>::into(num_substeps);
        self.inv_dt = 1.0 / self.dt;
    }

    fn compute_skinning_info(&mut self) {
        // create a hash for all vertices of the surface (visual) mesh
        let mut hash = Hash::new(SPACING, self.num_surface_verts);
        hash.create(&self.surface_pos);
        let mut min_dist = vec![f32::MAX; self.num_surface_verts];

        let mut tet_center;
        let mut mat;
        let mut bary;

        for i in 0..self.num_tets {
            tet_center = Vec3::ZERO;
            for j in 0..4 {
                tet_center += self.pos[self.tet_ids[i][j]] / 4.0;
            }

            let mut rmax: f32 = 0.0;
            for j in 0..4 {
                let r = tet_center.distance(self.pos[self.tet_ids[i][j]]);
                rmax = rmax.max(r);
            }
            rmax += SPACING;

            hash.query(&tet_center, rmax);
            if hash.query_size == 0 {
                continue;
            }

            let tet = self.tet_ids[i];
            let id0 = tet[0];
            let id1 = tet[1];
            let id2 = tet[2];
            let id3 = tet[3];

            mat = Mat3::from_cols(
                self.pos[id0] - self.pos[id3],
                self.pos[id1] - self.pos[id3],
                self.pos[id2] - self.pos[id3],
            );
            mat = mat.inverse();

            for j in 0..hash.query_size {
                let id = hash.query_ids[j];

                // we already have skinning info
                if min_dist[id] <= 0.0 {
                    continue;
                }

                if self.surface_pos[id].distance_squared(tet_center) > rmax * rmax {
                    continue;
                }

                // compute barycentric coords for candidate
                bary = self.surface_pos[id] - self.pos[id3];
                bary = mat * bary;
                let bary3 = 1.0 - bary[0] - bary[1] - bary[2];

                let mut dist: f32 = 0.0;
                for k in 0..3 {
                    dist = dist.max(-bary[k]);
                }
                dist = dist.max(-bary3);

                if dist < min_dist[id] {
                    min_dist[id] = dist;
                    self.skinning_info[id] = Some((i, [bary[0], bary[1], bary[2]]));
                }
            }
        }
    }

    pub fn reset(&mut self) {
        self.pos.copy_from_slice(&self.mesh.tet_vertices);
        self.surface_pos
            .copy_from_slice(&self.mesh.surface_vertices);
        self.prev.copy_from_slice(&self.pos);
        self.vel.fill(Vec3::ZERO);
    }

    fn init(&mut self) {
        for i in 0..self.num_tets {
            let vol = self.get_tet_volume(i);
            self.rest_vol[i] = vol;
            let inv_mass = if vol > 0.0 { 1.0 / (vol / 4.0) } else { 0.0 };
            let tet = self.tet_ids[i];
            self.inv_mass[tet[0]] += inv_mass;
            self.inv_mass[tet[1]] += inv_mass;
            self.inv_mass[tet[2]] += inv_mass;
            self.inv_mass[tet[3]] += inv_mass;
        }
        for i in 0..self.edge_lens.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            self.edge_lens[i] = self.pos[id0].distance(self.pos[id1]);
        }

        self.compute_skinning_info();
    }

    fn pre_solve(&mut self) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            self.vel[i] += GRAVITY * self.dt;
            self.prev[i] = self.pos[i];
            self.pos[i] += self.vel[i] * self.dt;
            if self.pos[i].y < 0.0 {
                self.pos[i] = self.prev[i];
                self.pos[i].y = 0.0;
            }
        }
    }

    fn solve(&mut self) {
        self.solve_edges();
        self.solve_volumes();
    }

    fn post_solve(&mut self) {
        for i in 0..self.num_particles {
            if self.inv_mass[i] == 0.0 {
                continue;
            }
            self.vel[i] = (self.pos[i] - self.prev[i]) * self.inv_dt;
        }
    }

    fn solve_edges(&mut self) {
        let alpha = self.edge_compliance * self.inv_dt * self.inv_dt;
        for i in 0..self.edge_lens.len() {
            let id0 = self.edge_ids[2 * i];
            let id1 = self.edge_ids[2 * i + 1];
            let w0 = self.inv_mass[id0];
            let w1 = self.inv_mass[id1];
            let w = w0 + w1;
            if w == 0.0 {
                continue;
            }

            let mut temp = self.pos[id0] - self.pos[id1];
            let len = temp.length();
            if len == 0.0 {
                continue;
            }
            temp /= len;
            let rest_len = self.edge_lens[i];
            let c = len - rest_len;
            let s = -c / (w + alpha);
            self.pos[id0] += temp * s * w0;
            self.pos[id1] += temp * -s * w1;
        }
    }

    fn solve_volumes(&mut self) {
        let alpha = self.vol_compliance * self.inv_dt * self.inv_dt;
        for i in 0..self.num_tets {
            let mut w = 0.0;
            let tet = self.tet_ids[i];
            let mut grads = [Vec3::ZERO; 4];
            for j in 0..4 {
                let order = VOL_ID_ORDER[j];
                let id0 = tet[order[0]];
                let id1 = tet[order[1]];
                let id2 = tet[order[2]];

                let temp0 = self.pos[id1] - self.pos[id0];
                let temp1 = self.pos[id2] - self.pos[id0];
                grads[j] = temp0.cross(temp1) / 6.0;
                w += self.inv_mass[tet[j]] * grads[j].length_squared();
            }
            if w == 0.0 {
                continue;
            }
            let vol = self.get_tet_volume(i);
            let rest_vol = self.rest_vol[i];
            let c = vol - rest_vol;
            let s = -c / (w + alpha);
            for (j, grad) in grads.iter().enumerate() {
                let id = self.tet_ids[i][j];
                self.pos[id] += *grad * s * self.inv_mass[id];
            }
        }
    }

    pub fn step(&mut self) {
        for _ in 0..self.num_substeps {
            self.pre_solve();
            self.solve();
            self.post_solve();
        }

        self.update_surface();
    }

    fn update_surface(&mut self) {
        for i in 0..self.num_surface_verts {
            if let Some((tetid, [b0, b1, b2])) = self.skinning_info[i] {
                let b3 = 1.0 - b0 - b1 - b2;
                let tet = self.tet_ids[tetid];
                let id0 = tet[0];
                let id1 = tet[1];
                let id2 = tet[2];
                let id3 = tet[3];
                self.surface_pos[i] = Vec3::ZERO;
                self.surface_pos[i] += self.pos[id0] * b0;
                self.surface_pos[i] += self.pos[id1] * b1;
                self.surface_pos[i] += self.pos[id2] * b2;
                self.surface_pos[i] += self.pos[id3] * b3;
            }
        }
    }

    #[must_use]
    fn get_tet_volume(&self, i: usize) -> f32 {
        let tet = self.tet_ids[i];
        let id0 = tet[0];
        let id1 = tet[1];
        let id2 = tet[2];
        let id3 = tet[3];
        let temp0 = self.pos[id1] - self.pos[id0];
        let temp1 = self.pos[id2] - self.pos[id0];
        let temp2 = self.pos[id3] - self.pos[id0];
        let temp3 = temp0.cross(temp1);
        temp3.dot(temp2) / 6.0
    }

    pub fn squash(&mut self) {
        for i in 0..self.num_particles {
            self.pos[i].y = SQUASH_TO_Y;
        }
        self.update_surface();
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
