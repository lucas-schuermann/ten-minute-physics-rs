use core::f64::consts::PI;

use glam::Vec2;
use js_sys::Math::random;
use wasm_bindgen::prelude::*;
use web_sys::CanvasRenderingContext2d;

const SIM_HEIGHT: f32 = 1.0;
//const DEFAULT_OBSTACLE_POS: Vec2 = Vec2::ZERO;
const DEFAULT_OBSTACLE_RADIUS: f32 = 0.2;
const DEFAULT_NUM_ITERS: usize = 10;
const DEFAULT_OVER_RELAXATION: f32 = 1.9;
const DEFAULT_TIMESTEP: f32 = 1.0 / 60.0;
const DEFAULT_SWIRL_PROBABILITY: f32 = 80.0; // 50.0
const SWIRL_MAX_RADIUS: f32 = 0.04; // 0.05
const MAX_NUM_SWIRLS: usize = 100;
const DEFAULT_NUM_CELLS: f32 = 100_000.0;

#[derive(Clone, Copy)]
enum Field {
    U,
    V,
    T,
}

#[wasm_bindgen]
pub struct FireSimulation {
    h: f32,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    pub num_iters: usize,
    pub over_relaxation: f32,

    obstacle_pos: Vec2,
    obstacle_radius: f32,
    pub burning_obstacle: bool,
    pub burning_floor: bool,

    num_cells_x: usize,
    num_cells_y: usize,
    #[wasm_bindgen(readonly)]
    pub num_cells: usize,
    u: Vec<f32>,
    v: Vec<f32>,
    new_u: Vec<f32>,
    new_v: Vec<f32>,
    s: Vec<f32>,
    t: Vec<f32>,
    new_t: Vec<f32>,

    pub swirl_probability: f32,
    num_swirls: usize,
    swirl_pos: Vec<Vec2>,
    swirl_omega: Vec<f32>,
    swirl_time: Vec<f32>,

    // rendering
    width: f32,
    height: f32,
    c_scale: f32,
    context: CanvasRenderingContext2d,
    pub show_obstacle: bool,
    pub show_swirls: bool,
}

fn set_color(dest: &mut [u8; 4], src: &[f32; 3]) {
    dest[0] = f32::floor(src[0]) as u8;
    dest[1] = f32::floor(src[1]) as u8;
    dest[2] = f32::floor(src[2]) as u8;
}

fn get_fire_color(val: f32) -> [f32; 3] {
    let val = f32::clamp(val, 0.0, 1.0);
    let r;
    let g;
    let b;
    if val < 0.3 {
        let s = val / 0.3;
        r = 0.2 * s;
        g = 0.2 * s;
        b = 0.2 * s;
    } else if val < 0.5 {
        let s = (val - 0.3) / 0.2;
        r = 0.2 + 0.8 * s;
        g = 0.1;
        b = 0.1;
    } else {
        let s = (val - 0.5) / 0.48;
        r = 1.0;
        g = s;
        b = 0.0;
    }
    return [255.0 * r, 255.0 * g, 255.0 * b];
}

#[wasm_bindgen]
impl FireSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(width: f32, height: f32, context: CanvasRenderingContext2d) -> FireSimulation {
        let width = width.floor();
        let height = height.floor();

        let domain_height = SIM_HEIGHT;
        let domain_width = domain_height / height * width;
        let h = f32::sqrt(domain_width * domain_height / DEFAULT_NUM_CELLS);
        let num_cells_x = f32::floor(domain_width / h) as usize + 2;
        let num_cells_y = f32::floor(domain_height / h) as usize + 2;
        let num_cells = num_cells_x * num_cells_y;

        // LVSTODO
        //         if (numX < numY) {
        //             scene.swirlProbability = 80.0;
        //             scene.swirlMaxRadius = 0.04;
        //         }
        //
        //         scene.obstacleX = 0.5 * numX * h;
        //         scene.obstacleY = 0.3 * numY * h;
        //         scene.showObstacle = scene.burningObstacle;
        let obstacle_pos = Vec2::new(0.5 * num_cells_x as f32 * h, 0.3 * num_cells_y as f32 * h);

        let fire = Self {
            h,
            dt: DEFAULT_TIMESTEP,
            num_iters: DEFAULT_NUM_ITERS,
            over_relaxation: DEFAULT_OVER_RELAXATION,

            obstacle_pos,
            obstacle_radius: DEFAULT_OBSTACLE_RADIUS,
            burning_obstacle: true,
            burning_floor: false,

            num_cells_x,
            num_cells_y,
            num_cells,
            u: vec![0.0; num_cells],
            v: vec![0.0; num_cells],
            new_u: vec![0.0; num_cells],
            new_v: vec![0.0; num_cells],
            s: vec![1.0; num_cells],
            t: vec![0.0; num_cells],
            new_t: vec![0.0; num_cells],

            num_swirls: 0,
            swirl_probability: DEFAULT_SWIRL_PROBABILITY,
            swirl_pos: vec![Vec2::ZERO; MAX_NUM_SWIRLS],
            swirl_omega: vec![0.0; MAX_NUM_SWIRLS],
            swirl_time: vec![0.0; MAX_NUM_SWIRLS],

            // rendering
            width,
            height,
            c_scale: height / domain_height,
            context,
            show_obstacle: true,
            show_swirls: false,
        };

        fire
    }

    fn solve_incompressibility(&mut self) {
        let n = self.num_cells_y;
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
            Field::T => {
                dx = h2;
                dy = h2;
                &self.t
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

        for i in 1..self.num_cells_x {
            for j in 1..self.num_cells_y {
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

    fn advect_temperature(&mut self) {
        self.new_t.copy_from_slice(&self.t);

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

                    self.new_t[i * n + j] = self.sample_field(x, y, Field::T);
                }
            }
        }

        self.t.copy_from_slice(&self.new_t);
    }

    fn update_fire(&mut self) {
        let dt = self.dt;
        let h = self.h;
        let swirl_time_span = 1.0;
        let swirl_omega = 20.0;
        let swirl_damping = 10.0 * self.dt;
        let swirl_probability = self.swirl_probability * h * h;

        let fire_cooling = 1.2 * self.dt;
        let smoke_cooling = 0.3 * self.dt;
        let lift = 3.0;
        let acceleration = 6.0 * self.dt;
        let kernel_radius = SWIRL_MAX_RADIUS;

        // update swirls
        let n = self.num_cells_y;
        let max_x = (self.num_cells_x - 1) as f32 * self.h;
        let max_y = (self.num_cells_y - 1) as f32 * self.h;

        // kill swirls
        let mut num = 0;
        for i in 0..self.num_swirls {
            self.swirl_time[i] -= self.dt;
            if self.swirl_time[i] > 0.0 {
                self.swirl_time[num] = self.swirl_time[i];
                self.swirl_pos[num] = self.swirl_pos[i];
                self.swirl_omega[num] = self.swirl_omega[i];
                num += 1;
            }
        }
        self.num_swirls = num;

        // advect and modify velocity field
        for i in 0..self.num_swirls {
            //let age_scale = self.swirl_time[i] / swirl_time_span;
            let mut x = self.swirl_pos[i][0];
            let mut y = self.swirl_pos[i][1];
            let swirl_u = (1.0 - swirl_damping) * self.sample_field(x, y, Field::U);
            let swirl_v = (1.0 - swirl_damping) * self.sample_field(x, y, Field::V);
            x += swirl_u * dt;
            y += swirl_v * dt;
            x = x.clamp(h, max_x);
            y = y.clamp(h, max_y);

            self.swirl_pos[i] = Vec2::new(x, y);
            let omega = self.swirl_omega[i];

            // update surrounding velocity field
            let x0 = f32::max(f32::floor((x - kernel_radius) / h), 0.0) as usize;
            let y0 = f32::max(f32::floor((y - kernel_radius) / h), 0.0) as usize;
            let x1 = f32::min(
                f32::floor((x + kernel_radius) / h) + 1.0,
                self.num_cells_x as f32 - 1.0,
            ) as usize;
            let y1 = f32::min(
                f32::floor((y + kernel_radius) / h) + 1.0,
                self.num_cells_y as f32 - 1.0,
            ) as usize;
            for i in x0..=x1 {
                for j in y0..=y1 {
                    for dim in 0..2 {
                        let (vx, vy) = match dim {
                            0 => (i as f32 * h, (j as f32 + 0.5) * h),
                            1 => ((i as f32 + 0.5) * h, j as f32 * h),
                            _ => unreachable!(),
                        };
                        let rx = vx - x; // LVSTODO use vec?
                        let ry = vy - y;
                        let r = f32::sqrt(rx * rx + ry * ry);

                        if r < kernel_radius {
                            let mut s = 1.0;
                            if r > 0.8 * kernel_radius {
                                s = 5.0 - 5.0 / kernel_radius * r;
                                // s = (kernel_radius - r) / kernel_radius * age_scale;
                            }
                            if dim == 0 {
                                let target = ry * omega + swirl_u;
                                let u = self.u[n * i + j];
                                self.u[n * i + j] = (target - u) * s; // +=? LVSTODO
                            } else {
                                let target = -rx * omega + swirl_v;
                                let v = self.v[n * i + j];
                                self.v[n * i + j] += (target - v) * s;
                            }
                        }
                    }
                }
            }
        }

        // update temperatures
        let min_r = 0.85 * self.obstacle_radius;
        let max_r = self.obstacle_radius + h;

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let t = self.t[i * n + j];

                let cooling = if t < 0.3 { smoke_cooling } else { fire_cooling };
                self.t[i * n + j] = f32::max(t - cooling, 0.0);
                // let u = self.u[i * n + j]; // LVSTODO: should be unused?
                let v = self.v[i * n + j];
                let target_v = t * lift;
                self.v[i * n + j] += (target_v - v) * acceleration;

                let mut num_new_swirls = 0;

                // obstacle burning
                if self.burning_obstacle {
                    let dx = (i as f32 + 0.5) * h - self.obstacle_pos[0];
                    let dy = (j as f32 + 0.5) * h - self.obstacle_pos[1] - 3.0 * h;
                    let d = dx * dx + dy * dy;
                    if min_r * min_r <= d && d < max_r * max_r {
                        self.t[i * n + j] = 1.0;
                        if (random() as f32) < 0.5 * swirl_probability {
                            num_new_swirls += 1;
                        }
                    }
                }

                // floor burning
                if j < 4 && self.burning_floor {
                    self.t[i * n + j] = 1.0;
                    self.u[i * n + j] = 0.0;
                    self.v[i * n + j] = 0.0;
                    if (random() as f32) < swirl_probability {
                        num_new_swirls += 1;
                    }
                }

                for _ in 0..num_new_swirls {
                    if self.num_swirls >= MAX_NUM_SWIRLS {
                        break;
                    }
                    let nr = self.num_swirls;
                    self.swirl_pos[nr] = Vec2::new(i as f32 * h, j as f32 * h);
                    self.swirl_omega[nr] = (-1.0 + 2.0 * (random() as f32)) * swirl_omega;
                    self.swirl_time[nr] = swirl_time_span;
                    self.num_swirls += 1;
                }
            }
        }

        // smooth temperatures
        for i in 1..self.num_cells_x - 1 {
            for j in 1..self.num_cells_y - 1 {
                let t = self.t[i * n + j];
                if t == 1.0 {
                    let avg = (self.t[(i - 1) * n + (j - 1)]
                        + self.t[(i + 1) * n + (j - 1)]
                        + self.t[(i + 1) * n + (j + 1)]
                        + self.t[(i - 1) * n + (j + 1)])
                        * 0.25;
                    self.t[i * n + j] = avg;
                }
            }
        }
    }

    pub fn step(&mut self) {
        self.solve_incompressibility();
        self.extrapolate();
        self.advect_vel();
        self.advect_temperature();
        self.update_fire();
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

    pub fn draw_buffer(&mut self, render_buffer: &mut [u8]) {
        let h = self.h;
        let cx = f32::floor(self.c_scale * h) as usize + 1;
        let cy = f32::floor(self.c_scale * h) as usize + 1;
        let n = self.num_cells_y;

        let mut color = [255; 4];

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let t = self.t[i * n + j];
                let fire_color = get_fire_color(t);
                set_color(&mut color, &fire_color); // LVSTODO: combine?

                let x = f32::floor(self.c_x((i as f32 - 1.0) * h)) as usize;
                let y = f32::floor(self.c_y((j as f32 + 1.0) * h)) as usize;
                for yi in y..y + cy {
                    let mut p = 4 * (yi * self.width as usize + x);
                    for _ in 0..cx {
                        p += 4;
                        // y-coord extrema are cut off
                        if p <= render_buffer.len() {
                            render_buffer[p - 4..p].copy_from_slice(&color);
                        }
                    }
                }
            }
        }
    }

    pub fn draw_canvas(&mut self) {
        let c: &CanvasRenderingContext2d = &self.context;
        let obstacle_color_hex: JsValue = JsValue::from("#404040");
        let swirl_color_hex: JsValue = JsValue::from("#303030");

        if self.show_obstacle {
            let r = self.obstacle_radius + self.h;
            let o = self.obstacle_pos;

            c.set_line_width(20.0);
            c.set_stroke_style(&obstacle_color_hex);
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

        if self.show_swirls {
            let r = SWIRL_MAX_RADIUS;
            for i in 0..self.num_swirls {
                let p = self.swirl_pos[i];

                c.set_line_width(1.0);
                c.set_stroke_style(&swirl_color_hex);
                c.begin_path();
                c.begin_path();
                c.arc(
                    self.c_x(p[0]).into(),
                    self.c_y(p[1]).into(),
                    (self.c_scale * r).into(),
                    0.0,
                    2.0 * PI,
                )
                .unwrap(); // known safe
                c.close_path();
                c.stroke();
            }
        }
    }

    fn set_obstacle(&mut self, pos: Vec2, reset: bool) {
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
                    //self.s[i * n + j] = 0.0;
                    // LVSTODO move to const
                    self.u[i * n + j] += 0.2 * v.x;
                    self.u[(i + 1) * n + j] += 0.2 * v.x;
                    self.v[i * n + j] += 0.2 * v.y;
                    self.v[i * n + (j + 1)] += 0.2 * v.y;
                }
            }
        }
    }

    pub fn set_obstacle_from_canvas(&mut self, c_x: f32, c_y: f32, reset: bool) {
        let x = c_x / self.c_scale;
        let y = (self.height - c_y) / self.c_scale;
        let pos = Vec2::new(x, y);
        self.set_obstacle(pos, reset);
    }
}
