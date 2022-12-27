use glam::Vec3;
use wasm_bindgen::prelude::*;

use solver::self_collision_15::*;

#[wasm_bindgen]
pub struct SelfCollisionSimulation {
    cloth: Cloth,
    attach: bool,
}

#[wasm_bindgen]
impl SelfCollisionSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<SelfCollisionSimulation, JsValue> {
        let cloth = Cloth::new();
        Ok(Self {
            cloth,
            attach: false,
        })
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.cloth.reset(self.attach);
    }

    #[wasm_bindgen]
    pub fn set_attach(&mut self, attach: bool) {
        self.attach = attach;
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
        self.cloth.dt
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
        self.cloth.set_solver_substeps(num_substeps);
    }

    #[wasm_bindgen]
    pub fn set_handle_collisions(&mut self, handle_collisions: bool) {
        self.cloth.handle_collisions = handle_collisions
    }

    #[wasm_bindgen]
    pub fn set_bending_compliance(&mut self, compliance: f32) {
        self.cloth.bending_compliance = compliance;
    }

    #[wasm_bindgen]
    pub fn set_stretch_compliance(&mut self, compliance: f32) {
        self.cloth.stretch_compliance = compliance;
    }

    #[wasm_bindgen]
    pub fn set_shear_compliance(&mut self, compliance: f32) {
        self.cloth.shear_compliance = compliance;
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
        self.cloth.simulate();
    }
}
