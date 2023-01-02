use glam::Vec2;
use wasm_bindgen::prelude::*;

use solver::fluid_sim_17::*;

#[wasm_bindgen]
pub struct FluidSimulation {
    scene_type: SceneType,
    state: State,
}

#[wasm_bindgen]
#[derive(PartialEq)]
pub enum SceneType {
    WindTunnel,
    HiresTunnel,
    Tank,
    Paint,
}

#[wasm_bindgen]
impl FluidSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(scene_type: SceneType) -> FluidSimulation {
        let resolution: f32 = match scene_type {
            SceneType::Tank => 50.0,
            SceneType::HiresTunnel => 200.0,
            _ => 100.0,
        };
        let domain_height = 1.0;
        let domain_width = domain_height / SIM_HEIGHT * SIM_WIDTH;
        let h = domain_height / resolution;
        let num_cells_x = f32::floor(domain_width / h) as usize;
        let num_cells_y = f32::floor(domain_height / h) as usize;
        let params = Parameters {
            density: 1000.0,
            h,
            gravity: -9.81,
            dt: 1.0 / 60.0,
            num_iters: 40,
            over_relaxation: 1.9,
        };
        let state = State::new(params, num_cells_x, num_cells_y);
        let mut scene = Self { scene_type, state };
        match scene.scene_type {
            SceneType::Tank => scene.setup_tank(),
            SceneType::WindTunnel | SceneType::HiresTunnel => scene.setup_tunnel(),
            SceneType::Paint => scene.setup_paint(),
        }

        scene
    }

    fn setup_tank(&mut self) {
        let n = self.state.num_cells_y;
        for i in 0..self.state.num_cells_x {
            for j in 0..self.state.num_cells_y {
                let mut s = 1.0; // fluid
                if i == 0 || i == self.state.num_cells_x - 1 || j == 0 {
                    s = 0.0; // solid
                }
                self.state.s[i * n + j] = s;
            }
        }

        self.state.renderer.clear();
        self.state.renderer.show_pressure = true;
    }

    fn setup_tunnel(&mut self) {
        let n = self.state.num_cells_y;
        let input_velocity = 2.0;
        for i in 0..self.state.num_cells_x {
            for j in 0..self.state.num_cells_y {
                let mut s = 1.0; // fluid
                if i == 0 || j == 0 || j == self.state.num_cells_y - 1 {
                    s = 0.0; //solid
                }
                self.state.s[i * n + j] = s;
                if i == 1 {
                    self.state.u[i * n + j] = input_velocity;
                }
            }
        }

        let pipe_height = 0.1 * self.state.num_cells_y as f32;
        let min_j = f32::floor(0.5 * self.state.num_cells_y as f32 - 0.5 * pipe_height) as usize;
        let max_j = f32::floor(0.5 * self.state.num_cells_y as f32 + 0.5 * pipe_height) as usize;

        for j in min_j..max_j {
            self.state.m[j] = 0.0; //solid
        }

        // set obstacle radius?
        self.state.set_obstacle(Vec2::new(0.4, 0.5), true, false);

        self.state.params.gravity = 0.0;

        self.state.renderer.clear();
        self.state.renderer.show_smoke = true;

        if self.scene_type == SceneType::HiresTunnel {
            self.state.params.dt = 1.0 / 120.0;
            self.state.params.num_iters = 100;

            self.state.renderer.show_pressure = true;
        }
    }

    fn setup_paint(&mut self) {
        self.state.params.gravity = 0.0;
        self.state.params.over_relaxation = 1.0;
        self.state.obstacle_radius = 0.1;

        self.state.renderer.clear();
        self.state.renderer.show_smoke = true;
        self.state.renderer.show_smoke_gradient = true;
    }

    #[wasm_bindgen]
    pub fn set_obstacle(&mut self, x: f32, y: f32, reset: bool, modulate: bool) {
        self.state.set_obstacle(Vec2::new(x, y), reset, modulate);
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn get_h(&self) -> f32 {
        self.state.params.h
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn density(&self) -> f32 {
        self.state.params.density
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn num_iters(&self) -> usize {
        self.state.params.num_iters
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn over_relaxation(&self) -> f32 {
        self.state.params.over_relaxation
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn num_cells(&self) -> usize {
        self.state.num_cells
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn num_cells_x(&self) -> usize {
        self.state.num_cells_x
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn num_cells_y(&self) -> usize {
        self.state.num_cells_y
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn m_ptr(&self) -> *const f32 {
        self.state.m.as_ptr()
    }

    #[must_use]
    #[wasm_bindgen]
    pub fn render_buffer_ptr(&self) -> *const u8 {
        self.state.renderer.render_buffer.as_ptr()
    }

    #[wasm_bindgen]
    pub fn step(&mut self) {
        self.state.simulate();
    }
}
