use glam::Vec3;
use rand::Rng;
use wasm_bindgen::prelude::*;

use solver::softbodies_10::SoftBody;

#[wasm_bindgen]
pub struct SoftBodiesSimulation {
    bodies: Vec<SoftBody>,
    num_substeps: usize,
    edge_compliance: f32,
    vol_compliance: f32,
}

#[wasm_bindgen]
impl SoftBodiesSimulation {
    #[wasm_bindgen(constructor)]
    pub fn new(
        num_substeps: usize,
        edge_compliance: f32,
        vol_compliance: f32,
    ) -> SoftBodiesSimulation {
        let mut sim = Self {
            bodies: vec![],
            num_substeps,
            edge_compliance,
            vol_compliance,
        };
        sim.reset();
        sim
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.bodies.clear();
        self.bodies.push(SoftBody::new(
            self.num_substeps,
            self.edge_compliance,
            self.vol_compliance,
        ));
    }

    #[wasm_bindgen]
    pub fn add_body(&mut self) {
        let mut rng = rand::thread_rng();
        let displacement = Vec3::new(
            -1.0 + 2.0 * rng.gen::<f32>(),
            0.0,
            -1.0 + 2.0 * rng.gen::<f32>(),
        );
        let mut body = SoftBody::new(self.num_substeps, self.edge_compliance, self.vol_compliance);
        body.translate(displacement);
        self.bodies.push(body);
    }

    #[wasm_bindgen]
    pub fn squash(&mut self) {
        self.bodies.iter_mut().for_each(SoftBody::squash);
    }

    #[wasm_bindgen]
    pub fn num_particles_per_body(&self) -> usize {
        self.bodies[0].num_particles
    }

    #[wasm_bindgen]
    pub fn num_tets(&self) -> usize {
        self.bodies.iter().map(|b| b.num_tets).sum()
    }

    #[wasm_bindgen]
    pub fn dt(&self) -> f32 {
        self.bodies[0].dt
    }

    #[wasm_bindgen]
    pub fn start_grab(&mut self, id: usize, pos: &[f32]) {
        self.bodies[id].start_grab(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn move_grabbed(&mut self, id: usize, pos: &[f32]) {
        self.bodies[id].move_grabbed(&Vec3::from_slice(pos));
    }

    #[wasm_bindgen]
    pub fn end_grab(&mut self, id: usize, vel: &[f32]) {
        self.bodies[id].end_grab(&Vec3::from_slice(vel));
    }

    #[wasm_bindgen]
    pub fn particle_positions_ptr(&self, id: usize) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.bodies[id].pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn set_solver_substeps(&mut self, num_substeps: usize) {
        self.bodies
            .iter_mut()
            .for_each(|b| b.set_solver_substeps(num_substeps));
    }

    #[wasm_bindgen]
    pub fn set_edge_compliance(&mut self, compliance: f32) {
        self.bodies
            .iter_mut()
            .for_each(|b| b.edge_compliance = compliance);
    }

    #[wasm_bindgen]
    pub fn set_volume_compliance(&mut self, compliance: f32) {
        self.bodies
            .iter_mut()
            .for_each(|b| b.vol_compliance = compliance);
    }

    #[wasm_bindgen]
    pub fn surface_tri_ids(&self) -> Vec<usize> {
        // NOTE: this heap allocates for the return value!
        self.bodies[0].surface_tri_ids()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.bodies.iter_mut().for_each(SoftBody::simulate);
    }
}
