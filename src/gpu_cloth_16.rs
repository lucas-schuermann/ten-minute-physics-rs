use glam::{vec3, Vec3};
use wasm_bindgen::prelude::*;

const NUM_X: usize = 250;
const NUM_Y: usize = 250;
const CLOTH_Y: f32 = 2.2;
const CLOTH_SPACING: f32 = 0.01;
const SPHERE_CENTER: Vec3 = vec3(0.0, 1.5, 0.0);
const SPHERE_RADIUS: f32 = 0.3;

#[wasm_bindgen]
extern "C" {
    // Use `js_namespace` here to bind `console.log(..)` instead of just
    // `log(..)`
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}

enum SolverType {
    Coloring,
    Jacobi,
}

struct SolverPass {
    size: f32,
    independent: bool,
}

#[wasm_bindgen]
pub struct GPUClothSimulation {
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    //     #[wasm_bindgen(readonly)]
    //     pub num_tris: usize,
    //     #[wasm_bindgen(readonly)]
    //     pub num_substeps: u8,
    //     #[wasm_bindgen(readonly)]
    //     pub dt: f32,
    //     inv_dt: f32,
    //
    //     edge_ids: Vec<[usize; 2]>,
    //     tri_ids: Vec<[usize; 3]>,
    //
    //     pos: Vec<Vec3>,
    //     prev: Vec<Vec3>,
    //     rest: Vec<Vec3>,
    //     inv_mass: Vec<f32>,
    //     corr: Vec<Vec3>,
    //     vel: Vec<Vec3>,
    //     normals: Vec<Vec3>,
    //
    //     passes: Vec<SolverPass>,
    //
    //     grab_particle_id: usize,
    //     grab_depth: f32,
    //     grab_inv_mass: f32,
    //
    //     pub solver_type: SolverType,
    //     pub jacobi_scale: f32,
}

#[wasm_bindgen]
impl GPUClothSimulation {
    #[allow(clippy::new_without_default)]
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new() -> GPUClothSimulation {
        let sim = Self { num_particles: 1 };
        sim
    }
}
