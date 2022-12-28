use glam::Vec3;
use wasm_bindgen::prelude::*;

use solver::softbodies_10::*;

#[wasm_bindgen]
pub struct SoftBodiesSimulation {
    body: SoftBody,
}

#[wasm_bindgen]
impl SoftBodiesSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<SoftBodiesSimulation, JsValue> {
        let body = SoftBody::new();
        Ok(Self { body })
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        //self.bodies.clear();
        // LVSTODO add one body
    }

    #[wasm_bindgen]
    pub fn num_particles(&self) -> usize {
        self.body.num_particles
        //self.bodies.iter().map(|b| b.num_particles).sum()
    }

    #[wasm_bindgen]
    pub fn num_tets(&self) -> usize {
        self.body.num_tets
        //self.bodies.iter().map(|b| b.num_tets).sum()
    }

    #[wasm_bindgen]
    pub fn dt(&self) -> f32 {
        self.body.dt
    }

    #[wasm_bindgen]
    pub fn start_grab(&mut self, pos: &[f32]) {
        self.body.start_grab(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn move_grabbed(&mut self, pos: &[f32]) {
        self.body.move_grabbed(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn end_grab(&mut self, vel: &[f32]) {
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

    #[wasm_bindgen]
    pub fn tet_surface_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        SoftBody::tet_surface_tri_ids()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.body.simulate();
    }
}
