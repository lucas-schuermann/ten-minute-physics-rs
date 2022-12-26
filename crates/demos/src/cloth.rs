use glam::Vec3;
use wasm_bindgen::prelude::*;

use solver::cloth::State;

#[wasm_bindgen]
pub struct ClothSim {
    state: State,
}

#[wasm_bindgen]
impl ClothSim {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<ClothSim, JsValue> {
        Ok(Self {
            state: State::new(),
        })
    }

    #[wasm_bindgen]
    pub fn num_particles(&self) -> usize {
        self.state.num_particles()
    }

    #[wasm_bindgen]
    pub fn num_tris(&self) -> usize {
        self.state.tri_ids().len()
    }

    #[wasm_bindgen]
    pub fn dt(&self) -> f32 {
        self.state.dt
    }

    #[wasm_bindgen]
    pub fn start_grab(&mut self, pos: &[f32]) {
        self.state.body.start_grab(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn move_grabbed(&mut self, pos: &[f32]) {
        self.state.body.move_grabbed(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn end_grab(&mut self, vel: &[f32]) {
        self.state.body.end_grab(&Vec3::from_slice(vel));
    }

    #[wasm_bindgen]
    pub fn particle_positions_ptr(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.state.particle_positions_ptr()
    }

    #[wasm_bindgen]
    pub fn set_solver_substeps(&mut self, num_substeps: usize) {
        self.state.set_solver_substeps(num_substeps);
    }

    #[wasm_bindgen]
    pub fn set_bending_compliance(&mut self, compliance: f32) {
        self.state.body.bending_compliance = compliance;
    }

    #[wasm_bindgen]
    pub fn set_stretching_compliance(&mut self, compliance: f32) {
        self.state.body.stretching_compliance = compliance;
    }

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen]
    pub fn mesh_edge_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.state
            .edge_ids()
            .iter()
            .map(|e| e.to_vec())
            .flatten()
            .collect()
    }

    #[wasm_bindgen]
    pub fn mesh_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.state
            .tri_ids()
            .iter()
            .map(|e| e.to_vec())
            .flatten()
            .collect()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.state.update();
    }
}
