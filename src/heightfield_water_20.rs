use std::f32::consts::PI;

use glam::{vec3, Vec3};
use wasm_bindgen::prelude::*;

const GRAVITY: Vec3 = vec3(0.0, -10.0, 0.0);
const TIME_STEP: f32 = 1.0 / 30.0;
const WAVE_SPEED: f32 = 2.0;
const POS_DAMPING: f32 = 1.0;
const VEL_DAMPING: f32 = 0.3;
const ALPHA: f32 = 0.5;
const RESTITUTION: f32 = 0.1;
const ITERS: usize = 2;

pub struct Ball {
    pub pos: Vec3,
    vel: Vec3,
    mass: f32,
    pub radius: f32,
    restitution: f32,
    grabbed: bool,
}

impl Ball {
    pub fn new(pos: Vec3, radius: f32, density: f32) -> Self {
        Self {
            pos,
            vel: Vec3::ZERO,
            mass: 4.0 * PI / 3.0 * radius.powi(3) * density,
            radius,
            restitution: RESTITUTION,
            grabbed: false,
        }
    }

    pub fn handle_collision(&mut self, other: &mut Ball) {
        let mut dir = other.pos - self.pos;
        let d = dir.length();

        let min_dist = self.radius + other.radius;
        if d >= min_dist {
            return;
        }

        dir *= 1.0 / d;
        let corr = (min_dist - d) / 2.0;
        self.pos += dir * -corr;
        other.pos += dir * corr;

        let v1 = self.vel.dot(dir);
        let v2 = other.vel.dot(dir);

        let m1 = self.mass;
        let m2 = other.mass;

        let nv1 = (m1 * v1 + m2 * v2 - m2 * (v1 - v2) * self.restitution) / (m1 + m2);
        let nv2 = (m1 * v1 + m2 * v2 - m1 * (v2 - v1) * self.restitution) / (m1 + m2);

        self.vel += dir * (nv1 - v1);
        other.vel += dir * (nv2 - v2);
    }

    pub fn step(&mut self, dt: f32, size_x: f32, size_z: f32, border: f32) {
        if self.grabbed {
            return;
        }

        self.vel += GRAVITY * dt;
        self.pos += self.vel * dt;

        let wx = 0.5 * size_x - self.radius - 0.5 * border;
        let wz = 0.5 * size_z - self.radius - 0.5 * border;

        if self.pos.x < -wx {
            self.pos.x = -wx;
            self.vel.x = -self.restitution * self.vel.x;
        }
        if self.pos.x > wx {
            self.pos.x = wx;
            self.vel.x = -self.restitution * self.vel.x;
        }
        if self.pos.z < -wz {
            self.pos.z = -wz;
            self.vel.z = -self.restitution * self.vel.z;
        }
        if self.pos.z > wz {
            self.pos.z = wz;
            self.vel.z = -self.restitution * self.vel.z;
        }
        if self.pos.y < self.radius {
            self.pos.y = self.radius;
            self.vel.y = -self.restitution * self.vel.y;
        }
    }

    pub fn apply_force_y(&mut self, dt: f32, force: f32) {
        self.vel.y += dt * force / self.mass;
        self.vel *= 0.999; // TOOD: move to constant
    }

    pub fn start_grab(&mut self, pos: &Vec3) {
        self.grabbed = true;
        self.pos = *pos;
    }

    pub fn move_grabbed(&mut self, pos: &Vec3) {
        self.pos = *pos;
    }

    pub fn end_grab(&mut self, vel: &Vec3) {
        self.grabbed = false;
        self.vel = *vel;
    }
}

#[wasm_bindgen]
pub struct HeightFieldWaterSimulation {
    #[wasm_bindgen(readonly)]
    pub num_x: usize,
    #[wasm_bindgen(readonly)]
    pub num_z: usize,
    #[wasm_bindgen(readonly)]
    pub num_cells: usize,

    // surface
    spacing: f32,
    heights: Vec<f32>,
    body_heights: Vec<f32>,
    prev_heights: Vec<f32>,
    velocities: Vec<f32>,
    wave_speed: f32,

    positions: Vec<Vec3>,
    uvs: Vec<f32>,
    indices: Vec<usize>,

    balls: Vec<Ball>,
    boundary_x: f32,
    boundary_z: f32,
    boundary_size: f32,
}

#[wasm_bindgen]
impl HeightFieldWaterSimulation {
    #[allow(clippy::new_without_default)]
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(
        size_x: f32,
        size_z: f32,
        depth: f32,
        spacing: f32,
        boundary_size: f32,
    ) -> HeightFieldWaterSimulation {
        let num_x = f32::floor(size_x / spacing) as usize + 1;
        let num_z = f32::floor(size_z / spacing) as usize + 1;
        let num_cells = num_x * num_z;

        let mut uvs = vec![0.0; num_cells * 2];
        for i in 0..num_x {
            for j in 0..num_z {
                uvs[2 * (i * num_z + j)] = i as f32 / num_x as f32;
                uvs[2 * (i * num_z + j) + 1] = j as f32 / num_z as f32;
            }
        }

        let mut indices = vec![0; (num_x - 1) * (num_z - 1) * 2 * 3];
        let mut pos = 0;
        for i in 0..(num_x - 1) {
            for j in 0..(num_z - 1) {
                let id0 = i * num_z + j;
                let id1 = i * num_z + j + 1;
                let id2 = (i + 1) * num_z + j + 1;
                let id3 = (i + 1) * num_z + j;

                indices[pos] = id0;
                indices[pos + 1] = id1;
                indices[pos + 2] = id2;
                pos += 3;

                indices[pos] = id0;
                indices[pos + 1] = id2;
                indices[pos + 2] = id3;
                pos += 3;
            }
        }

        let mut sim = Self {
            num_x,
            num_z,
            num_cells,

            spacing,
            heights: vec![depth; num_cells],
            body_heights: vec![0.0; num_cells],
            prev_heights: vec![depth; num_cells],
            velocities: vec![0.0; num_cells],
            wave_speed: WAVE_SPEED,

            positions: vec![Vec3::ZERO; num_cells],
            uvs,
            indices,

            balls: vec![],
            boundary_x: size_x,
            boundary_z: size_z,
            boundary_size,
        };
        sim.reset(depth);
        sim
    }

    #[wasm_bindgen(getter)]
    pub fn positions(&self) -> *const Vec3 {
        // Generally, self is unsafe! We take care in JS to make sure to
        // query the positions array pointer after heap allocations have
        // occurred (which move the location).
        // Positions is a Vec<Vec3>, which is a linear array of f32s in
        // memory.
        self.positions.as_ptr()
    }

    #[wasm_bindgen(getter)]
    pub fn ball_radii(&self) -> Vec<f32> {
        self.balls.iter().map(|b| b.radius).collect()
    }

    #[wasm_bindgen(getter)]
    pub fn ball_positions(&self) -> Vec<f32> {
        self.balls
            .iter()
            .map(|b| b.pos.to_array())
            .flatten()
            .collect()
    }

    // We can copy since we are not performance sensitive for these two methods
    #[wasm_bindgen(getter)]
    pub fn uvs(&self) -> Vec<f32> {
        // NOTE: self heap allocates for the return value!
        self.uvs.clone()
    }

    #[wasm_bindgen(getter)]
    pub fn indices(&self) -> Vec<usize> {
        // NOTE: self heap allocates for the return value!
        self.indices.clone()
    }

    fn simulate_coupling(&mut self, dt: f32) {
        let cx = f32::floor(self.num_x as f32 / 2.0);
        let cz = f32::floor(self.num_z as f32 / 2.0);
        let h1 = 1.0 / self.spacing;

        self.prev_heights
            .copy_from_slice(self.body_heights.as_slice());
        self.body_heights.fill(0.0);

        for ball in &mut self.balls {
            let pos = ball.pos;
            let br = ball.radius;
            let h2 = self.spacing * self.spacing;

            let x0 = f32::max(0.0, cx + f32::floor((pos.x - br) * h1)) as usize;
            let x1 = f32::min((self.num_x - 1) as f32, cx + f32::floor((pos.x + br) * h1)) as usize;
            let z0 = f32::max(0.0, cz + f32::floor((pos.z - br) * h1)) as usize;
            let z1 = f32::min((self.num_z - 1) as f32, cz + f32::floor((pos.z + br) * h1)) as usize;

            for xi in x0..=x1 {
                for zi in z0..=z1 {
                    let x = (xi as f32 - cx) * self.spacing;
                    let z = (zi as f32 - cz) * self.spacing;
                    let r2 = (pos.x - x) * (pos.x - x) + (pos.z - z) * (pos.z - z);
                    if r2 < br * br {
                        let body_half_height = f32::sqrt(br * br - r2);
                        let water_height = self.heights[xi * self.num_z + zi];

                        let body_min = f32::max(pos.y - body_half_height, 0.0);
                        let body_max = f32::min(pos.y + body_half_height, water_height);
                        let body_height = f32::max(body_max - body_min, 0.0);
                        if body_height > 0.0 {
                            ball.apply_force_y(dt, -body_height * h2 * GRAVITY.y);
                            self.body_heights[xi * self.num_z + zi] += body_height;
                        }
                    }
                }
            }
        }

        for _ in 0..ITERS {
            for xi in 0..self.num_x {
                for zi in 0..self.num_z {
                    let id = xi * self.num_z + zi;

                    let mut num = if (xi > 0 && xi < self.num_x - 1) {
                        2.0
                    } else {
                        1.0
                    };
                    num += if (zi > 0 && zi < self.num_z - 1) {
                        2.0
                    } else {
                        1.0
                    };
                    let mut avg = 0.0;
                    if (xi > 0) {
                        avg += self.body_heights[id - self.num_z];
                    }
                    if (xi < self.num_x - 1) {
                        avg += self.body_heights[id + self.num_z];
                    }
                    if (zi > 0) {
                        avg += self.body_heights[id - 1];
                    }
                    if (zi < self.num_z - 1) {
                        avg += self.body_heights[id + 1];
                    }
                    avg /= num;
                    self.body_heights[id] = avg;
                }
            }
        }

        for i in 0..self.num_cells {
            let body_change = self.body_heights[i] - self.prev_heights[i];
            self.heights[i] += ALPHA * body_change;
        }
    }

    fn simulate_surface(&mut self, dt: f32) {
        self.wave_speed = f32::min(self.wave_speed, 0.5 * self.spacing / dt);
        let c = self.wave_speed * self.wave_speed / self.spacing / self.spacing;
        let pd = f32::min(POS_DAMPING * dt, 1.0);
        let vd = f32::max(0.0, 1.0 - VEL_DAMPING * dt);

        for i in 0..self.num_x {
            for j in 0..self.num_z {
                let id = i * self.num_z + j;
                let h = self.heights[id];
                let mut sum_h = 0.0;
                sum_h += if i > 0 {
                    self.heights[id - self.num_z]
                } else {
                    h
                };
                sum_h += if i < self.num_x - 1 {
                    self.heights[id + self.num_z]
                } else {
                    h
                };
                sum_h += if j > 0 { self.heights[id - 1] } else { h };
                sum_h += if j < self.num_z - 1 {
                    self.heights[id + 1]
                } else {
                    h
                };
                self.velocities[id] += dt * c * (sum_h - 4.0 * h);
                self.heights[id] += (0.25 * sum_h - h) * pd; // positional damping
            }
        }

        for i in 0..self.num_cells {
            self.velocities[i] *= vd; // velocity damping
            self.heights[i] += self.velocities[i] * dt;
        }
    }

    pub fn step(&mut self, dt: f32) {
        self.simulate_coupling(dt);
        self.simulate_surface(dt);

        for i in 0..self.balls.len() {
            self.balls[i].step(dt, self.boundary_x, self.boundary_z, self.boundary_size);
            for j in 0..i {
                //self.balls[i].handle_collision(&mut self.balls[j]); // TODO: borrow checker is dumb
            }
        }

        // TODO: should we just operate on positions directly?
        for (i, p) in self.positions.iter_mut().enumerate() {
            p.y = self.heights[i];
        }
    }

    pub fn reset(&mut self, depth: f32) {
        let cx = f32::floor(self.num_x as f32 / 2.0);
        let cz = f32::floor(self.num_z as f32 / 2.0);
        for i in 0..self.num_x {
            for j in 0..self.num_z {
                self.positions[i * self.num_z + j] = vec3(
                    (i as f32 - cx) * self.spacing,
                    depth,
                    (j as f32 - cz) * self.spacing,
                );
            }
        }

        self.balls.push(Ball::new(vec3(-0.5, 1.0, -0.5), 0.2, 2.0));
        self.balls.push(Ball::new(vec3(0.5, 1.0, -0.5), 0.3, 0.7));
        self.balls.push(Ball::new(vec3(0.5, 1.0, 0.5), 0.25, 0.2));
    }

    pub fn start_grab(&mut self, id: usize, pos: &[f32]) {
        self.balls[id].start_grab(&Vec3::from_slice(pos));
    }

    pub fn move_grabbed(&mut self, id: usize, pos: &[f32]) {
        self.balls[id].move_grabbed(&Vec3::from_slice(pos));
    }

    pub fn end_grab(&mut self, id: usize, vel: &[f32]) {
        self.balls[id].end_grab(&Vec3::from_slice(vel));
    }
}
