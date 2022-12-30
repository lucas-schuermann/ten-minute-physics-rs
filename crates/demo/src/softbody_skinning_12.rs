use glam::Vec3;
use wasm_bindgen::prelude::*;

use solver::softbody_skinning_12::SkinnedSoftbody;

#[wasm_bindgen]
pub struct SkinnedSoftbodySimulation {
    body: SkinnedSoftbody,
}

#[wasm_bindgen]
impl SkinnedSoftbodySimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        num_substeps: usize,
        edge_compliance: f32,
        vol_compliance: f32,
    ) -> SkinnedSoftbodySimulation {
        let body = SkinnedSoftbody::new(num_substeps, edge_compliance, vol_compliance);
        Self { body }
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.body.reset();
    }

    #[wasm_bindgen]
    pub fn squash(&mut self) {
        self.body.squash();
    }

    #[wasm_bindgen]
    pub fn num_particles(&self) -> usize {
        self.body.num_particles
    }

    #[wasm_bindgen]
    pub fn num_surface_verts(&self) -> usize {
        self.body.num_surface_verts
    }

    #[wasm_bindgen]
    pub fn num_tets(&self) -> usize {
        self.body.num_tets
    }

    pub fn num_tris(&self) -> usize {
        self.body.num_tris
    }

    #[wasm_bindgen]
    pub fn dt(&self) -> f32 {
        self.body.dt
    }

    #[wasm_bindgen]
    pub fn start_grab(&mut self, _: usize, pos: &[f32]) {
        self.body.start_grab(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn move_grabbed(&mut self, _: usize, pos: &[f32]) {
        self.body.move_grabbed(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn end_grab(&mut self, _: usize, vel: &[f32]) {
        self.body.end_grab(&Vec3::from_slice(vel));
    }

    #[wasm_bindgen]
    pub fn particle_positions_ptr(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.body.pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn surface_positions_ptr(&self) -> *const Vec3 {
        // See `self.particle_positions_ptr` for comment on safety
        self.body.surface_pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn set_solver_substeps(&mut self, num_substeps: usize) {
        self.body.set_solver_substeps(num_substeps);
    }

    #[wasm_bindgen]
    pub fn set_edge_compliance(&mut self, compliance: f32) {
        self.body.edge_compliance = compliance;
    }

    #[wasm_bindgen]
    pub fn set_volume_compliance(&mut self, compliance: f32) {
        self.body.vol_compliance = compliance;
    }

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen]
    pub fn tet_edge_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.body.edge_ids.clone()
    }

    #[wasm_bindgen]
    pub fn surface_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.body.surface_tri_ids()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.body.simulate();
    }
}
