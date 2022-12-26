mod cloth;
mod hash;

use glam::Vec3;
use wasm_bindgen::prelude::*;

pub enum DemoKind {
    Hash,
    Cloth,
    SoftBody,
}

pub trait Demo {
    //fn new() -> Result<Demo, JsValue>;
    fn step();
    fn reset();
}

#[wasm_bindgen]
pub struct HashSim {
    state: hash::Demo,
}

#[wasm_bindgen]
impl HashSim {
    #[wasm_bindgen(constructor)]
    pub fn new() -> Result<HashSim, JsValue> {
        let mut state = hash::Demo::new();
        state.init();
        Ok(Self { state })
    }

    #[wasm_bindgen]
    pub fn num_bodies(&self) -> usize {
        *hash::NUM_BODIES
    }

    #[wasm_bindgen]
    pub fn radius(&self) -> f32 {
        hash::RADIUS
    }

    #[wasm_bindgen]
    pub fn body_positions_ptr(&self) -> *const Vec3 {
        // Generally, this is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.state.pos.as_ptr()
    }

    #[wasm_bindgen]
    pub fn body_collisions_ptr(&self) -> *const u8 {
        // See above comment for `body_positions_ptr` re: safety
        self.state.collisions.as_ptr()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.state.update();
    }

    #[wasm_bindgen]
    pub fn reset(&mut self) {
        self.state.init();
    }
}
