use glam::Vec2;

pub const WINDOW_WIDTH: usize = 800;
pub const WINDOW_HEIGHT: usize = 600;
pub const SIM_HEIGHT: f32 = 1.1;
pub const SIM_WIDTH: f32 = WINDOW_WIDTH as f32 / WINDOW_HEIGHT as f32 / SIM_HEIGHT;
const C_SCALE: f32 = WINDOW_HEIGHT as f32 / SIM_HEIGHT;

#[derive(Clone, Copy)]
enum Field {
    U,
    V,
    S,
}

pub struct Parameters {
    pub density: f32,
    pub h: f32,
    pub gravity: f32,
    pub dt: f32,
    pub num_iters: usize,
    pub over_relaxation: f32,
}

pub struct Renderer {
    pub render_buffer: Vec<u8>,
    pub show_pressure: bool,
    pub show_smoke: bool,
    pub show_smoke_gradient: bool,
}

impl Renderer {
    pub fn new() -> Self {
        Renderer {
            render_buffer: vec![255; WINDOW_WIDTH * WINDOW_HEIGHT * 4], // rgba
            show_pressure: false,
            show_smoke: false,
            show_smoke_gradient: false,
        }
    }

    pub fn clear(&mut self) {
        self.render_buffer.fill(255);
        self.show_pressure = false;
        self.show_smoke = false;
        self.show_smoke_gradient = false;
    }
}

pub struct State {
    pub params: Parameters,

    pub obstacle_pos: Vec2,
    pub obstacle_radius: f32,
    frame_number: usize,

    pub num_cells_x: usize,
    pub num_cells_y: usize,
    pub num_cells: usize,
    pub u: Vec<f32>,
    v: Vec<f32>,
    new_u: Vec<f32>,
    new_v: Vec<f32>,
    p: Vec<f32>,
    pub s: Vec<f32>,
    pub m: Vec<f32>,
    new_m: Vec<f32>,

    pub renderer: Renderer,
}

#[inline(always)]
fn c_x(x: f32) -> f32 {
    return x * C_SCALE;
}

#[inline(always)]
fn c_y(y: f32) -> f32 {
    return WINDOW_HEIGHT as f32 - y * C_SCALE;
}

fn get_sci_color(val: f32, min: f32, max: f32) -> [f32; 4] {
    let mut val = val.clamp(min, max - 0.0001);
    let d = max - min;
    val = if d == 0.0 { 0.5 } else { (val - min) / d };
    let m = 0.25;
    let num = f32::floor(val / m);
    let s = (val - num * m) / m;
    let (r, g, b) = match num as usize {
        0 => (0.0, s, 1.0),
        1 => (0.0, 1.0, 1.0 - s),
        2 => (s, 1.0, 0.0),
        3 => (1.0, 1.0 - s, 0.0),
        _ => (0.0, 0.0, 0.0),
    };
    return [255.0 * r, 255.0 * g, 255.0 * b, 255.0];
}

impl State {
    #[must_use]
    pub fn new(params: Parameters, num_cells_x: usize, num_cells_y: usize) -> Self {
        let num_cells_x = num_cells_x + 2;
        let num_cells_y = num_cells_y + 2;
        let num_cells = num_cells_x * num_cells_y;
        Self {
            params,

            obstacle_pos: Vec2::ZERO,
            obstacle_radius: 0.15,
            frame_number: 0,

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

            renderer: Renderer::new(),
        }
    }

    fn integrate(&mut self) {
        let n = self.num_cells_y;
        for i in 1..self.num_cells_x {
            for j in 1..self.num_cells_y - 1 {
                if self.s[i * n + j] != 0.0 && self.s[i * n + j - 1] != 0.0 {
                    self.v[i * n + j] += self.params.gravity * self.params.dt;
                }
            }
        }
    }

    fn solve_incompressibility(&mut self) {
        let n = self.num_cells_y;
        let cp = self.params.density * self.params.h / self.params.dt;
        for _ in 0..self.params.num_iters {
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
                    let p = -div / s * self.params.over_relaxation;
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
        let h = self.params.h;
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

        let dt = self.params.dt;
        let n = self.num_cells_y;
        let h = self.params.h;
        let h2 = 0.5 * h;

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                // u component
                if self.s[i * n + j] != 0.0 && self.s[(i - 1) * n + j] != 0.0 && j < n - 1 {
                    let mut x = i as f32 * h;
                    let mut y = j as f32 * h + h2;
                    let mut u = self.u[i * n + j];
                    let v = self.avg_v(i, j);
                    x = x - dt * u;
                    y = y - dt * v;
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
                    x = x - dt * u;
                    y = y - dt * v;
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

        let dt = self.params.dt;
        let n = self.num_cells_y;
        let h = self.params.h;
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

    pub fn set_obstacle(&mut self, pos: Vec2, reset: bool, modulate: bool) {
        let mut v = Vec2::ZERO;

        if !reset {
            v = (pos - self.obstacle_pos) / self.params.dt;
        }

        self.obstacle_pos = pos;
        let r = self.obstacle_radius;
        let n = self.num_cells_y;
        let h = self.params.h;

        for i in 1..self.num_cells_x - 2 {
            for j in 1..self.num_cells_y - 2 {
                self.s[i * n + j] = 1.0;
                let dx = (i as f32 + 0.5) * h - pos.x;
                let dy = (j as f32 + 0.5) * h - pos.y;

                if dx * dx + dy * dy < r * r {
                    self.s[i * n + j] = 0.0;
                    self.m[i * n + j] = if modulate {
                        0.5 + 0.5 * f32::sin(0.1 * self.frame_number as f32)
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

    pub fn draw(&mut self) {
        let h = self.params.h;
        let cx = f32::floor(C_SCALE * 1.1 * h) as usize + 1;
        let cy = f32::floor(C_SCALE * 1.1 * h) as usize + 1;
        let n = self.num_cells_y;
        let mut color = [255; 4];

        let mut p_min = self.p[0];
        let mut p_max = self.p[0];
        if self.renderer.show_pressure {
            for i in 0..self.num_cells {
                p_min = f32::min(p_min, self.p[i]);
                p_max = f32::max(p_max, self.p[i]);
            }
        }

        for i in 0..self.num_cells_x {
            for j in 0..self.num_cells_y {
                let ind = i * n + j;
                if self.renderer.show_pressure {
                    let p = self.p[ind];
                    let s = self.m[ind];
                    let sci_color = get_sci_color(p, p_min, p_max);
                    if self.renderer.show_smoke {
                        color[0] = f32::max(0.0, sci_color[0] - 255.0 * s) as u8;
                        color[1] = f32::max(0.0, sci_color[1] - 255.0 * s) as u8;
                        color[2] = f32::max(0.0, sci_color[2] - 255.0 * s) as u8;
                    } else {
                        color[0] = sci_color[0] as u8;
                        color[1] = sci_color[1] as u8;
                        color[2] = sci_color[2] as u8;
                    }
                } else if self.renderer.show_smoke {
                    let s = self.m[ind];
                    if self.renderer.show_smoke_gradient {
                        let sci_color = get_sci_color(s, 0.0, 1.0);
                        color[0] = sci_color[0] as u8;
                        color[1] = sci_color[1] as u8;
                        color[2] = sci_color[2] as u8;
                    } else {
                        color[0..=2].fill((255.0 * s) as u8);
                    }
                } else if self.s[ind] == 0.0 {
                    color[0..=2].fill(0);
                }
                let x = f32::floor(c_x(i as f32 * h)) as usize;
                let y = f32::floor(c_y((j as f32 + 1.0) * h)) as usize;
                for yi in y..y + cy {
                    let mut p = 4 * (yi * WINDOW_WIDTH + x);
                    for _ in 0..cx {
                        // LVSTODO cleaner ways to loop
                        if p + 4 < self.renderer.render_buffer.len() {
                            self.renderer.render_buffer[p..p + 4].copy_from_slice(&color);
                        }
                        p += 4;
                    }
                }
            }
        }
    }

    pub fn simulate(&mut self) {
        self.integrate();
        self.p.fill(0.0);
        self.solve_incompressibility();
        self.extrapolate();
        self.advect_vel();
        self.advect_smoke();
        self.draw();
        self.frame_number += 1;
    }
}
