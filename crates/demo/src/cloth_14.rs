use glam::Vec3;
use wasm_bindgen::prelude::*;

use solver::cloth_14::*;

#[wasm_bindgen]
pub struct ClothSimulation {
    cloth: Cloth,
    num_substeps: usize,
    dt: f32,
}

#[wasm_bindgen]
impl ClothSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ClothSimulation, JsValue> {
        let dt = TIME_STEP / DEFAULT_NUM_SOLVER_SUBSTEPS as f32;
        let cloth = Cloth::new(dt);

        Ok(Self {
            cloth,
            num_substeps: DEFAULT_NUM_SOLVER_SUBSTEPS,
            dt,
        })
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.cloth.reset();
    }

    #[wasm_bindgen]
    pub fn num_particles(&self) -> usize {
        self.cloth.num_particles
    }

    #[wasm_bindgen]
    pub fn num_tris(&self) -> usize {
        self.cloth.tri_ids.len()
    }

    #[wasm_bindgen]
    pub fn dt(&self) -> f32 {
        self.dt
    }

    #[wasm_bindgen]
    pub fn start_grab(&mut self, pos: &[f32]) {
        self.cloth.start_grab(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn move_grabbed(&mut self, pos: &[f32]) {
        self.cloth.move_grabbed(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn end_grab(&mut self, vel: &[f32]) {
        self.cloth.end_grab(&Vec3::from_slice(vel));
    }

    #[wasm_bindgen]
    pub fn particle_positions_ptr(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.cloth.pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn set_solver_substeps(&mut self, num_substeps: usize) {
        self.num_substeps = num_substeps;
        self.dt = TIME_STEP / num_substeps as f32;
        self.cloth.set_dt(self.dt);
    }

    #[wasm_bindgen]
    pub fn set_bending_compliance(&mut self, compliance: f32) {
        self.cloth.bending_compliance = compliance;
    }

    #[wasm_bindgen]
    pub fn set_stretching_compliance(&mut self, compliance: f32) {
        self.cloth.stretching_compliance = compliance;
    }

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen]
    pub fn mesh_edge_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.cloth
            .edge_ids
            .iter()
            .map(|e| e.to_vec())
            .flatten()
            .collect()
    }

    #[wasm_bindgen]
    pub fn mesh_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.cloth
            .tri_ids
            .iter()
            .map(|e| e.to_vec())
            .flatten()
            .collect()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        for _ in 0..self.num_substeps {
            self.cloth.simulate();
        }
    }
}
