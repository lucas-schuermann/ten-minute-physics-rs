#![allow(clippy::many_single_char_names, clippy::similar_names)]

use std::f64::consts::PI;

use glam::Vec2;
use wasm_bindgen::prelude::*;
use wasm_bindgen::Clamped;
use web_sys::{CanvasRenderingContext2d, ImageData};

use crate::get_sci_color_255;

const SIM_HEIGHT: f32 = 1.0;

#[wasm_bindgen]
#[derive(PartialEq, Clone, Copy)]
pub enum SceneType {
    WindTunnel,
    HiresTunnel,
    Tank,
    Paint,
}

#[derive(Clone, Copy)]
enum Field {
    U,
    V,
    S,
}

#[allow(clippy::struct_excessive_bools)]
#[wasm_bindgen]
pub struct FluidSimulation {
    #[wasm_bindgen(readonly)]
    pub density: f32,
    h: f32,
    gravity: f32,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    pub num_iters: usize,
    pub over_relaxation: f32,

    obstacle_pos: Vec2,
    obstacle_radius: f32,
    frame_number: f32, // store as f32 to be used in sin modulation

    num_cells_x: usize,
    num_cells_y: usize,
    #[wasm_bindgen(readonly)]
    pub num_cells: usize,
    u: Vec<f32>,
    v: Vec<f32>,
    new_u: Vec<f32>,
    new_v: Vec<f32>,
    p: Vec<f32>,
    s: Vec<f32>,
    m: Vec<f32>,
    new_m: Vec<f32>,

    // rendering
    width: f32,
    height: f32,
    c_scale: f32,
    render_buffer: Vec<u8>,
    context: CanvasRenderingContext2d,
    pub show_obstacle: bool,
    pub show_streamlines: bool,
    pub show_velocities: bool,
    pub show_pressure: bool,
    pub show_smoke: bool,
    show_smoke_gradient: bool,
}

fn splat_color(color: &mut [u8; 4], val: f32) {
    let val = f32::floor(val) as u8;
    color[0..=2].fill(val);
}

fn set_color(dest: &mut [u8; 4], src: &[f32; 3]) {
    dest[0] = f32::floor(src[0]) as u8;
    dest[1] = f32::floor(src[1]) as u8;
    dest[2] = f32::floor(src[2]) as u8;
}

#[wasm_bindgen]
impl FluidSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(
        scene_type: SceneType,
        width: f32,
        height: f32,
        context: CanvasRenderingContext2d,
    ) -> FluidSimulation {
        let width = width.floor();
        let height = height.floor();

        let resolution: f32 = match scene_type {
            SceneType::Tank => 50.0,
            SceneType::HiresTunnel => 200.0,
            _ => 100.0,
        };
        let domain_height = SIM_HEIGHT;
        let domain_width = domain_height / height * width;
        let h = domain_height / resolution;
        let num_cells_x = f32::floor(domain_width / h) as usize + 2;
        let num_cells_y = f32::floor(domain_height / h) as usize + 2;
        let num_cells = num_cells_x * num_cells_y;

        let render_buffer = vec![255; width as usize * height as usize * 4]; // rgba
        let mut fluid = Self {
            density: 1000.0,
            h,
            gravity: -9.81,
            dt: 1.0 / 60.0,
            num_iters: 40,
            over_relaxation: 1.9,

            obstacle_pos: Vec2::ZERO,
            obstacle_radius: 0.15,
            frame_number: 0.0,

            num_cells_x,
            num_cells_y,
            num_cells,
            u: vec![0.0; num_cells],
            v: vec![0.0; num_cells],
            new_u: vec![0.0; num_cells],
            new_v: vec![0.0; num_cells],
            p: vec![0.0; num_cells],
            s: vec![0.0; num_cells],
            m: vec![1.0; num_cells],
            new_m: vec![0.0; num_cells],

            // rendering
            width,
            height,
            c_scale: height / domain_height,
            render_buffer,
            context,

            show_obstacle: true,
            show_streamlines: false,
            show_velocities: false,
            show_pressure: false,
            show_smoke: false,
            show_smoke_gradient: false,
        };

        match scene_type {
            SceneType::Tank => fluid.setup_tank(),
            SceneType::WindTunnel | SceneType::HiresTunnel => fluid.setup_tunnel(scene_type),
            SceneType::Paint => fluid.setup_paint(),
        }

        fluid
    }

    fn setup_tank(&mut self) {
        let n = self.num_cells_y;
        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let mut s = 1.0; // fluid
                if i == 0 || i == self.num_cells_x - 1 || j == 0 {
                    s = 0.0; // solid
                }
                self.s[i * n + j] = s;
            }
        }

        self.show_pressure = true;
        self.show_obstacle = false;
    }

    fn setup_tunnel(&mut self, scene_type: SceneType) {
        let n = self.num_cells_y;
        let input_velocity = 2.0;
        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let mut s = 1.0; // fluid
                if i == 0 || j == 0 || j == self.num_cells_y - 1 {
                    s = 0.0; //solid
                }
                self.s[i * n + j] = s;
                if i == 1 {
                    self.u[i * n + j] = input_velocity;
                }
            }
        }

        let pipe_height = 0.1 * self.num_cells_y as f32;
        let min_j = f32::floor(0.5 * self.num_cells_y as f32 - 0.5 * pipe_height) as usize;
        let max_j = f32::floor(0.5 * self.num_cells_y as f32 + 0.5 * pipe_height) as usize;

        for j in min_j..max_j {
            self.m[j] = 0.0; // solid
        }

        self.set_obstacle(Vec2::new(0.4, 0.5), true, false);

        self.gravity = 0.0;

        self.show_smoke = true;

        if scene_type == SceneType::HiresTunnel {
            self.dt = 1.0 / 120.0;
            self.num_iters = 100;

            self.show_pressure = true;
        }
    }

    fn setup_paint(&mut self) {
        self.gravity = 0.0;
        self.over_relaxation = 1.0;
        self.obstacle_radius = 0.1;

        self.show_smoke = true;
        self.show_smoke_gradient = true;
        self.show_obstacle = false;
    }

    fn integrate(&mut self) {
        let n = self.num_cells_y;
        for i in 1..self.num_cells_x {
            for j in 1..self.num_cells_y - 1 {
                if self.s[i * n + j] != 0.0 && self.s[i * n + j - 1] != 0.0 {
                    self.v[i * n + j] += self.gravity * self.dt;
                }
            }
        }
    }

    fn solve_incompressibility(&mut self) {
        let n = self.num_cells_y;
        let cp = self.density * self.h / self.dt;
        for _ in 0..self.num_iters {
            for i in 1..self.num_cells_x - 1 {
                for j in 1..self.num_cells_y - 1 {
                    if self.s[i * n + j] == 0.0 {
                        continue;
                    }

                    let sx0 = self.s[(i - 1) * n + j];
                    let sx1 = self.s[(i + 1) * n + j];
                    let sy0 = self.s[i * n + j - 1];
                    let sy1 = self.s[i * n + j + 1];
                    let s = sx0 + sx1 + sy0 + sy1;
                    if s == 0.0 {
                        continue;
                    }

                    let div = self.u[(i + 1) * n + j] - self.u[i * n + j] + self.v[i * n + j + 1]
                        - self.v[i * n + j];
                    let p = -div / s * self.over_relaxation;
                    self.p[i * n + j] += cp * p;

                    self.u[i * n + j] -= sx0 * p;
                    self.u[(i + 1) * n + j] += sx1 * p;
                    self.v[i * n + j] -= sy0 * p;
                    self.v[i * n + j + 1] += sy1 * p;
                }
            }
        }
    }

    fn extrapolate(&mut self) {
        let n = self.num_cells_y;
        for i in 0..self.num_cells_x {
            self.u[i * n] = self.u[i * n + 1];
            self.u[i * n + n - 1] = self.u[i * n + n - 2];
        }
        for j in 0..self.num_cells_y {
            self.v[j] = self.v[n + j];
            self.v[(self.num_cells_x - 1) * n + j] = self.v[(self.num_cells_x - 2) * n + j];
        }
    }

    #[must_use]
    fn sample_field(&self, x: f32, y: f32, field: Field) -> f32 {
        let n = self.num_cells_y;
        let h = self.h;
        let h1 = 1.0 / h;
        let h2 = 0.5 * h;
        let x = x.clamp(h, self.num_cells_x as f32 * h);
        let y = y.clamp(h, self.num_cells_y as f32 * h);

        let mut dx = 0.0;
        let mut dy = 0.0;
        let f = match field {
            Field::U => {
                dy = h2;
                &self.u
            }
            Field::V => {
                dx = h2;
                &self.v
            }
            Field::S => {
                dx = h2;
                dy = h2;
                &self.m
            }
        };

        let x0 = f32::min(f32::floor((x - dx) * h1), (self.num_cells_x - 1) as f32) as usize;
        let tx = ((x - dx) - x0 as f32 * h) * h1;
        let x1 = usize::min(x0 + 1, self.num_cells_x - 1);

        let y0 = f32::min(f32::floor((y - dy) * h1), (self.num_cells_y - 1) as f32) as usize;
        let ty = ((y - dy) - y0 as f32 * h) * h1;
        let y1 = usize::min(y0 + 1, self.num_cells_y - 1);

        let sx = 1.0 - tx;
        let sy = 1.0 - ty;

        sx * sy * f[x0 * n + y0]
            + tx * sy * f[x1 * n + y0]
            + tx * ty * f[x1 * n + y1]
            + sx * ty * f[x0 * n + y1]
    }

    #[must_use]
    fn avg_u(&self, i: usize, j: usize) -> f32 {
        let n = self.num_cells_y;
        (self.u[i * n + j - 1]
            + self.u[i * n + j]
            + self.u[(i + 1) * n + j - 1]
            + self.u[(i + 1) * n + j])
            * 0.25
    }

    #[must_use]
    fn avg_v(&self, i: usize, j: usize) -> f32 {
        let n = self.num_cells_y;
        (self.v[(i - 1) * n + j]
            + self.v[i * n + j]
            + self.v[(i - 1) * n + j + 1]
            + self.v[i * n + j + 1])
            * 0.25
    }

    fn advect_vel(&mut self) {
        self.new_u.copy_from_slice(&self.u);
        self.new_v.copy_from_slice(&self.v);

        let dt = self.dt;
        let n = self.num_cells_y;
        let h = self.h;
        let h2 = 0.5 * h;

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                // u component
                if self.s[i * n + j] != 0.0 && self.s[(i - 1) * n + j] != 0.0 && j < n - 1 {
                    let mut x = i as f32 * h;
                    let mut y = j as f32 * h + h2;
                    let mut u = self.u[i * n + j];
                    let v = self.avg_v(i, j);
                    x -= dt * u;
                    y -= dt * v;
                    u = self.sample_field(x, y, Field::U);
                    self.new_u[i * n + j] = u;
                }
                // v component
                if self.s[i * n + j] != 0.0
                    && self.s[i * n + j - 1] != 0.0
                    && i < self.num_cells_x - 1
                {
                    let mut x = i as f32 * h + h2;
                    let mut y = j as f32 * h;
                    let u = self.avg_u(i, j);
                    let mut v = self.v[i * n + j];
                    x -= dt * u;
                    y -= dt * v;
                    v = self.sample_field(x, y, Field::V);
                    self.new_v[i * n + j] = v;
                }
            }
        }

        self.u.copy_from_slice(&self.new_u);
        self.v.copy_from_slice(&self.new_v);
    }

    fn advect_smoke(&mut self) {
        self.new_m.copy_from_slice(&self.m);

        let dt = self.dt;
        let n = self.num_cells_y;
        let h = self.h;
        let h2 = 0.5 * h;

        for i in 1..self.num_cells_x - 1 {
            for j in 1..self.num_cells_y - 1 {
                if self.s[i * n + j] != 0.0 {
                    let u = (self.u[i * n + j] + self.u[(i + 1) * n + j]) * 0.5;
                    let v = (self.v[i * n + j] + self.v[i * n + j + 1]) * 0.5;
                    let x = i as f32 * h + h2 - dt * u;
                    let y = j as f32 * h + h2 - dt * v;

                    self.new_m[i * n + j] = self.sample_field(x, y, Field::S);
                }
            }
        }
        self.m.copy_from_slice(&self.new_m);
    }

    fn set_obstacle(&mut self, pos: Vec2, reset: bool, modulate: bool) {
        let mut v = Vec2::ZERO;

        if !reset {
            v = (pos - self.obstacle_pos) / self.dt;
        }

        self.obstacle_pos = pos;
        let r = self.obstacle_radius;
        let n = self.num_cells_y;
        let h = self.h;

        for i in 1..self.num_cells_x - 2 {
            for j in 1..self.num_cells_y - 2 {
                self.s[i * n + j] = 1.0;
                let dx = (i as f32 + 0.5) * h - pos.x;
                let dy = (j as f32 + 0.5) * h - pos.y;

                if dx * dx + dy * dy < r * r {
                    self.s[i * n + j] = 0.0;
                    self.m[i * n + j] = if modulate {
                        0.5 + 0.5 * f32::sin(0.1 * self.frame_number)
                    } else {
                        1.0
                    };
                    self.u[i * n + j] = v.x;
                    self.u[(i + 1) * n + j] = v.x;
                    self.v[i * n + j] = v.y;
                    self.v[i * n + (j + 1)] = v.y;
                }
            }
        }
    }

    pub fn set_obstacle_from_canvas(&mut self, c_x: f32, c_y: f32, reset: bool, modulate: bool) {
        let x = c_x / self.c_scale;
        let y = (self.height - c_y) / self.c_scale;
        let pos = Vec2::new(x, y);
        self.set_obstacle(pos, reset, modulate);
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn c_x(&self, x: f32) -> f32 {
        x * self.c_scale
    }

    #[allow(clippy::inline_always)]
    #[inline(always)]
    fn c_y(&self, y: f32) -> f32 {
        self.height - y * self.c_scale
    }

    #[allow(clippy::too_many_lines)]
    pub fn draw(&mut self) {
        let h = self.h;
        let cx = f32::floor(self.c_scale * h) as usize + 1;
        let cy = f32::floor(self.c_scale * h) as usize + 1;
        let n = self.num_cells_y;
        let mut color = [255; 4];

        let mut p_min = self.p[0];
        let mut p_max = self.p[0];
        if self.show_pressure {
            for i in 0..self.num_cells {
                p_min = f32::min(p_min, self.p[i]);
                p_max = f32::max(p_max, self.p[i]);
            }
        }

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let ind = i * n + j;
                if self.show_pressure {
                    let p = self.p[ind];
                    let s = self.m[ind];
                    let sci_color = get_sci_color_255(p, p_min, p_max);
                    if self.show_smoke {
                        set_color(
                            &mut color,
                            &[
                                f32::max(0.0, sci_color[0] - 255.0 * s),
                                f32::max(0.0, sci_color[1] - 255.0 * s),
                                f32::max(0.0, sci_color[2] - 255.0 * s),
                            ],
                        );
                    } else {
                        set_color(&mut color, &sci_color);
                    }
                } else if self.show_smoke {
                    let s = self.m[ind];
                    if self.show_smoke_gradient {
                        let sci_color = get_sci_color_255(s, 0.0, 1.0);
                        set_color(&mut color, &sci_color);
                    } else {
                        splat_color(&mut color, 255.0 * s);
                    }
                } else if self.s[ind] == 0.0 {
                    color[0..=2].fill(0);
                }
                let x = f32::floor(self.c_x((i as f32 - 1.0) * h)) as usize;
                let y = f32::floor(self.c_y((j as f32 + 1.0) * h)) as usize;
                for yi in y..y + cy {
                    let mut p = 4 * (yi * self.width as usize + x);
                    for _ in 0..cx {
                        // LVSTODO cleaner ways to loop
                        if p + 3 < self.render_buffer.len() {
                            self.render_buffer[p..p + 4].copy_from_slice(&color);
                        }
                        p += 4;
                    }
                }
            }
        }

        let c = &self.context;

        let image_data = ImageData::new_with_u8_clamped_array_and_sh(
            Clamped(&self.render_buffer),
            self.width as u32,
            self.height as u32,
        )
        .expect("failed to create image data mapped to render buffer");
        c.put_image_data(&image_data, 0.0, 0.0)
            .expect("failed to write render buffer to canvas");

        if self.show_velocities {
            c.set_stroke_style(&JsValue::from("#000000"));
            // LVSTODO move to consts
            let scale = 0.02;
            let h = self.h;
            let n = self.num_cells_y;

            for i in 0..self.num_cells_x {
                for j in 0..self.num_cells_y {
                    let u = self.u[i * n + j];
                    let v = self.v[i * n + j];

                    let i = i as f32;
                    let j = j as f32;

                    c.begin_path();
                    let x0: f64 = self.c_x(i * h).into();
                    let x1: f64 = self.c_x(i * h + u * scale).into();
                    let y: f64 = self.c_y((j + 0.5) * h).into();
                    c.move_to(x0, y);
                    c.line_to(x1, y);
                    c.stroke();

                    c.begin_path();
                    let x: f64 = self.c_x((i + 0.5) * h).into();
                    let y0: f64 = self.c_y(j * h).into();
                    let y1: f64 = self.c_y(j * h + v * scale).into();
                    c.move_to(x, y0);
                    c.line_to(x, y1);
                    c.stroke();
                }
            }
        }

        if self.show_streamlines {
            // LVSTODO move to consts
            let seg_len = self.h * 0.2;
            let num_segs = 15;
            c.set_stroke_style(&JsValue::from("#000000"));

            for i in (1..(self.num_cells_x - 1)).step_by(5) {
                for j in (1..(self.num_cells_y - 1)).step_by(5) {
                    let mut x = (i as f32 + 0.5) * self.h;
                    let mut y = (j as f32 + 0.5) * self.h;
                    c.begin_path();
                    c.move_to(self.c_x(x).into(), self.c_y(y).into());
                    for _ in 0..num_segs {
                        let u = self.sample_field(x, y, Field::U);
                        let v = self.sample_field(x, y, Field::V);
                        let l = f32::sqrt(u * u + v * v);
                        x += u / l * seg_len;
                        y += v / l * seg_len;
                        x += u * 0.01;
                        y += v * 0.01;
                        if x > self.num_cells_x as f32 * self.h {
                            break;
                        }
                        c.line_to(self.c_x(x).into(), self.c_y(y).into());
                    }
                    c.stroke();
                }
            }
        }

        if self.show_obstacle {
            // LVSTODO move to consts
            let r = self.obstacle_radius + self.h;
            let o = self.obstacle_pos;
            if self.show_pressure {
                c.set_stroke_style(&JsValue::from("#000000"));
            } else {
                c.set_stroke_style(&JsValue::from("#DDDDDD"));
            }

            c.set_fill_style(&JsValue::from("#FFFFFF"));
            c.begin_path();
            c.arc(
                self.c_x(o[0]).into(),
                self.c_y(o[1]).into(),
                (self.c_scale * r).into(),
                0.0,
                2.0 * PI,
            )
            .unwrap(); // known safe
            c.close_path();
            c.fill();

            c.set_line_width(3.0);
            c.set_stroke_style(&JsValue::from("#000000"));
            c.begin_path();
            c.arc(
                self.c_x(o[0]).into(),
                self.c_y(o[1]).into(),
                (self.c_scale * r).into(),
                0.0,
                2.0 * PI,
            )
            .unwrap(); // known safe
            c.close_path();
            c.stroke();
            c.set_line_width(1.0);
        }
    }

    pub fn step(&mut self) {
        self.integrate();

        self.p.fill(0.0);
        self.solve_incompressibility();

        self.extrapolate();
        self.advect_vel();
        self.advect_smoke();

        self.frame_number += 1.0;
    }
}
