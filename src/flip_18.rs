#![allow(
    clippy::many_single_char_names,
    clippy::similar_names,
    clippy::cast_possible_wrap
)]

use std::f32::consts::PI;

use glam::{vec3, UVec2, Vec2, Vec3};
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlUniformLocation};

use crate::util::{compile_shader, get_sci_color, link_program, set_buffers_and_attributes};

const SIM_HEIGHT: f32 = 3.0;
const DEFAULT_RESOLUTION: f32 = 100.0;
const DAM_BREAK_REL_WATER_HEIGHT: f32 = 0.8;
const DAM_BREAK_REL_WATER_WIDTH: f32 = 0.6;
const PARTICLE_CELL_SCALE: f32 = 0.3;
const PARTICLE_SPACING_SCALE: f32 = 2.2;
const DEFAULT_OBSTACLE_RADIUS: f32 = 0.15;
const DEFAULT_DENSITY: f32 = 1000.0;
const DEFAULT_NUM_SUBSTEPS: usize = 1;
const DEFAULT_NUM_PRESSURE_ITERS: usize = 50;
const DEFAULT_NUM_PARTICLE_ITERS: usize = 2;
const DEFAULT_FLIP_RATIO: f32 = 0.9;
const DEFAULT_OVER_RELAXATION: f32 = 1.9;
const DEFAULT_GRAVITY: f32 = -9.81;
const DEFAULT_DT: f32 = 1.0 / 60.0;

const COLOR_DIFFUSION_COEFF: f32 = 0.001;
const PARTICLE_COLOR: Vec3 = vec3(0.0, 0.0, 1.0);
const OBSTALCE_COLOR: Vec3 = vec3(1.0, 0.0, 0.0);
const OBSTACLE_DISK_NUM_SEGS: usize = 100;
const GRID_POINT_SIZE_SCALE: f32 = 0.9;
const PARTICLE_POINT_SIZE_SCALE: f32 = 2.0;

#[derive(Clone, Copy, PartialEq, Eq)]
enum CellKind {
    Fluid,
    Air,
    Solid,
}

#[allow(clippy::struct_excessive_bools)]
#[wasm_bindgen]
pub struct FlipSimulation {
    #[wasm_bindgen(readonly)]
    pub density: f32,
    h: f32,
    gravity: f32,
    #[wasm_bindgen(readonly)]
    pub dt: f32,
    pub num_substeps: usize,
    num_pressure_iters: usize,
    num_particle_iters: usize,
    pub flip_ratio: f32,
    pub over_relaxation: f32,
    pub compensate_drift: bool,
    pub separate_particles: bool,

    obstacle_pos: Vec2,
    obstacle_vel: Vec2,
    obstacle_radius: f32,

    particle_num_cells_x: usize,
    particle_num_cells_y: usize,
    #[wasm_bindgen(readonly)]
    pub particle_num_cells: usize,
    #[wasm_bindgen(readonly)]
    pub num_particles: usize,
    particle_inv_spacing: f32,
    particle_radius: f32,
    particle_pos: Vec<Vec2>,
    particle_vel: Vec<Vec2>,
    particle_density: Vec<f32>,
    particle_rest_density: f32,
    num_cells_x: usize,
    num_cells_y: usize,
    #[wasm_bindgen(readonly)]
    pub num_cells: usize,
    inv_spacing: f32,
    u: Vec<f32>,
    v: Vec<f32>,
    du: Vec<f32>,
    dv: Vec<f32>,
    prev_u: Vec<f32>,
    prev_v: Vec<f32>,
    p: Vec<f32>,
    s: Vec<f32>,
    cell_num_particles: Vec<usize>,
    cell_first_particle: Vec<usize>,
    cell_particle_ids: Vec<usize>,
    cell_kind: Vec<CellKind>,

    // rendering
    particle_color: Vec<Vec3>,
    cell_color: Vec<Vec3>,
    width: f32,
    height: f32,
    c_scale: f32,
    pub show_obstacle: bool,
    pub show_particles: bool,
    pub show_grid: bool,
    renderer: WebGLRenderer,
}

struct WebGLRenderer {
    context: WebGl2RenderingContext,

    particle_program: WebGlProgram,
    particle_buffer: WebGlBuffer,
    particle_color_buffer: WebGlBuffer,
    grid_buffer: WebGlBuffer,
    grid_color_buffer: WebGlBuffer,
    particle_position_attrib_location: u32,
    particle_color_attrib_location: u32,
    particle_point_size_uniform: WebGlUniformLocation,
    particle_domain_size_uniform: WebGlUniformLocation,
    particle_mode_draw_disk_uniform: WebGlUniformLocation,

    mesh_program: WebGlProgram,
    disk_buffer: WebGlBuffer,
    disk_id_buffer: WebGlBuffer,
    mesh_position_attrib_location: u32,
    mesh_domain_size_uniform: WebGlUniformLocation,
    mesh_color_uniform: WebGlUniformLocation,
    mesh_translation_uniform: WebGlUniformLocation,
    mesh_scale_uniform: WebGlUniformLocation,
}

#[wasm_bindgen]
impl FlipSimulation {
    #[allow(clippy::too_many_lines)]
    #[wasm_bindgen(constructor)]
    pub fn new(
        width: f32,
        height: f32,
        context: WebGl2RenderingContext,
    ) -> Result<FlipSimulation, JsValue> {
        let width = width.floor();
        let height = height.floor();

        let domain_height = SIM_HEIGHT;
        let domain_width = domain_height / height * width;
        let h = domain_height / DEFAULT_RESOLUTION;
        let num_cells_x = f32::floor(domain_width / h) as usize + 2;
        let num_cells_y = f32::floor(domain_height / h) as usize + 2;
        let num_cells = num_cells_x * num_cells_y;
        let inv_spacing = 1.0 / h;

        let particle_radius = PARTICLE_CELL_SCALE * h;
        let particle_inv_spacing = 1.0 / (PARTICLE_SPACING_SCALE * particle_radius);
        let particle_num_cells_x = f32::floor(domain_width * particle_inv_spacing) as usize + 1;
        let particle_num_cells_y = f32::floor(domain_height * particle_inv_spacing) as usize + 1;
        let particle_num_cells = particle_num_cells_x * particle_num_cells_y;
        let dx = 2.0 * particle_radius;
        let dy = f32::sqrt(3.0) / 2.0 * dx;
        let num_particles_x = f32::floor(
            (DAM_BREAK_REL_WATER_WIDTH * domain_width - 2.0 * h - 2.0 * particle_radius) / dx,
        ) as usize;
        let num_particles_y = f32::floor(
            (DAM_BREAK_REL_WATER_HEIGHT * domain_height - 2.0 * h - 2.0 * particle_radius) / dy,
        ) as usize;
        let num_particles = num_particles_x * num_particles_y;

        let renderer = WebGLRenderer::new(
            context,
            width as i32,
            height as i32,
            num_cells_x,
            num_cells_y,
            h,
            num_particles,
        )?;

        let mut fluid = Self {
            density: DEFAULT_DENSITY,
            h,
            gravity: DEFAULT_GRAVITY,
            dt: DEFAULT_DT,
            num_substeps: DEFAULT_NUM_SUBSTEPS,
            num_pressure_iters: DEFAULT_NUM_PRESSURE_ITERS,
            num_particle_iters: DEFAULT_NUM_PARTICLE_ITERS,
            flip_ratio: DEFAULT_FLIP_RATIO,
            over_relaxation: DEFAULT_OVER_RELAXATION,
            compensate_drift: true,
            separate_particles: true,

            obstacle_pos: Vec2::ZERO,
            obstacle_vel: Vec2::ZERO,
            obstacle_radius: DEFAULT_OBSTACLE_RADIUS,

            num_particles,
            particle_num_cells_x,
            particle_num_cells_y,
            particle_num_cells,
            particle_inv_spacing,
            particle_radius,
            particle_pos: vec![Vec2::ZERO; num_particles],
            particle_vel: vec![Vec2::ZERO; num_particles],
            particle_density: vec![0.0; num_particles],
            particle_rest_density: 0.0,
            cell_num_particles: vec![0; particle_num_cells],
            cell_first_particle: vec![0; particle_num_cells + 1],
            cell_particle_ids: vec![0; num_particles],
            num_cells_x,
            num_cells_y,
            num_cells,
            inv_spacing,
            u: vec![0.0; num_cells],
            v: vec![0.0; num_cells],
            du: vec![0.0; num_cells],
            dv: vec![0.0; num_cells],
            prev_u: vec![0.0; num_cells],
            prev_v: vec![0.0; num_cells],
            p: vec![0.0; num_cells],
            s: vec![0.0; num_cells],
            cell_kind: vec![CellKind::Air; num_cells],

            // rendering
            width,
            height,
            c_scale: height / domain_height,
            particle_color: vec![PARTICLE_COLOR; num_particles],
            cell_color: vec![Vec3::ZERO; num_cells],

            show_obstacle: true,
            show_particles: true,
            show_grid: false,
            renderer,
        };

        // create particles
        let mut p = 0;
        for i in 0..num_particles_x {
            for j in 0..num_particles_y {
                fluid.particle_pos[p] = Vec2::new(
                    h + particle_radius
                        + dx * i as f32
                        + (if j % 2 == 0 { 0.0 } else { particle_radius }),
                    h + particle_radius + dy * j as f32,
                );
                p += 1;
            }
        }

        // setup grid cells for fluid domain
        let n = fluid.num_cells_y;
        for i in 0..fluid.num_cells_x {
            for j in 0..fluid.num_cells_y {
                let mut s = 1.0; // fluid
                if i == 0 || i == fluid.num_cells_x - 1 || j == 0 {
                    s = 0.0; // solid
                }
                fluid.s[i * n + j] = s;
            }
        }

        // move obstacle out of the way for dam break
        fluid.set_obstacle(Vec2::new(domain_width * 0.6, domain_height * 0.5), true);

        Ok(fluid)
    }

    fn integrate_particles(&mut self) {
        for i in 0..self.num_particles {
            self.particle_vel[i].y += self.dt * self.gravity;
            self.particle_pos[i] += self.particle_vel[i] * self.dt;
        }
    }

    fn pos_to_cell_idx(&self, x: Vec2, particle_grid: bool) -> usize {
        let (spacing, num_x, num_y) = if particle_grid {
            (
                self.particle_inv_spacing,
                self.particle_num_cells_x,
                self.particle_num_cells_y,
            )
        } else {
            (self.inv_spacing, self.num_cells_x, self.num_cells_y)
        };

        let xi = UVec2::clamp(
            Vec2::floor(x * spacing).as_uvec2(),
            UVec2::ZERO,
            UVec2::new(num_x as u32 - 1, num_y as u32 - 1),
        );
        xi.x as usize * num_y + xi.y as usize
    }

    fn push_particles_apart(&mut self) {
        // count particles per cell
        self.cell_num_particles.fill(0);
        for i in 0..self.num_particles {
            let cell_idx = self.pos_to_cell_idx(self.particle_pos[i], true);
            self.cell_num_particles[cell_idx] += 1;
        }

        // partial sums
        let mut first = 0;
        for i in 0..self.particle_num_cells {
            first += self.cell_num_particles[i];
            self.cell_first_particle[i] = first;
        }
        self.cell_first_particle[self.particle_num_cells] = first; // guard

        // fill particles into cells
        for i in 0..self.num_particles {
            let cell_idx = self.pos_to_cell_idx(self.particle_pos[i], true);
            self.cell_first_particle[cell_idx] -= 1;
            self.cell_particle_ids[self.cell_first_particle[cell_idx]] = i;
        }

        // push particles apart
        let min_dist = 2.0 * self.particle_radius;
        let min_dist_sq = min_dist * min_dist;
        for _ in 0..self.num_particle_iters {
            for i in 0..self.num_particles {
                let p = self.particle_pos[i];
                let pxi = f32::floor(p.x * self.particle_inv_spacing);
                let pyi = f32::floor(p.y * self.particle_inv_spacing);
                let x0 = f32::max(pxi - 1.0, 0.0) as usize;
                let y0 = f32::max(pyi - 1.0, 0.0) as usize;
                let x1 = f32::min(pxi + 1.0, self.particle_num_cells_x as f32 - 1.0) as usize;
                let y1 = f32::min(pyi + 1.0, self.particle_num_cells_y as f32 - 1.0) as usize;

                for xi in x0..=x1 {
                    for yi in y0..=y1 {
                        let cell_idx = xi * self.particle_num_cells_y + yi;
                        let first = self.cell_first_particle[cell_idx];
                        let last = self.cell_first_particle[cell_idx + 1];
                        for j in first..last {
                            let id = self.cell_particle_ids[j];
                            if id == i {
                                continue;
                            }
                            let q = self.particle_pos[id];
                            let mut d = q - p;
                            let dist_sq = d.length_squared();
                            if dist_sq > min_dist_sq || dist_sq == 0.0 {
                                continue;
                            }
                            let dist = f32::sqrt(dist_sq);
                            let s = 0.5 * (min_dist - dist) / dist;
                            d *= s;
                            self.particle_pos[i] -= d;
                            self.particle_pos[id] += d;

                            // diffuse colors
                            for k in 0..3 {
                                let color0 = self.particle_color[i][k];
                                let color1 = self.particle_color[id][k];
                                let color = (color0 + color1) * 0.5;
                                self.particle_color[i][k] =
                                    color0 + (color - color0) * COLOR_DIFFUSION_COEFF;
                                self.particle_color[id][k] =
                                    color1 + (color - color1) * COLOR_DIFFUSION_COEFF;
                            }
                        }
                    }
                }
            }
        }
    }

    fn handle_particle_collisions(&mut self) {
        let h = self.h;
        let r = self.particle_radius;
        let min_dist = self.obstacle_radius + r;
        let min_dist_sq = min_dist * min_dist;

        let min_x = h + r;
        let max_x = (self.num_cells_x as f32 - 1.0) * h - r;
        let min_y = h + r;
        let max_y = (self.num_cells_y as f32 - 1.0) * h - r;

        for i in 0..self.num_particles {
            let mut x = self.particle_pos[i];
            let d = x - self.obstacle_pos;
            let dist_sq = d.length_squared();

            // obstacle collision
            if dist_sq < min_dist_sq {
                self.particle_vel[i] = self.obstacle_vel;
            }

            // wall collisions
            if x.x < min_x {
                x.x = min_x;
                self.particle_vel[i].x = 0.0;
            }
            if x.x > max_x {
                x.x = max_x;
                self.particle_vel[i].x = 0.0;
            }
            if x.y < min_y {
                x.y = min_y;
                self.particle_vel[i].y = 0.0;
            }
            if x.y > max_y {
                x.y = max_y;
                self.particle_vel[i].y = 0.0;
            }
            self.particle_pos[i] = x;
        }
    }

    #[allow(clippy::too_many_lines)]
    fn transfer_velocities(&mut self, to_grid: bool) {
        let n = self.num_cells_y;
        let h = self.h;
        let h1 = self.inv_spacing;
        let h2 = 0.5 * h;

        let nx = self.num_cells_x as f32;
        let ny = self.num_cells_y as f32;

        if to_grid {
            self.prev_u.copy_from_slice(&self.u);
            self.prev_v.copy_from_slice(&self.v);
            self.du.fill(0.0);
            self.dv.fill(0.0);
            self.u.fill(0.0);
            self.v.fill(0.0);

            for i in 0..self.num_cells {
                self.cell_kind[i] = if self.s[i] == 0.0 {
                    CellKind::Solid
                } else {
                    CellKind::Air
                };
            }

            for i in 0..self.num_particles {
                let cell_idx = self.pos_to_cell_idx(self.particle_pos[i], false);
                if self.cell_kind[cell_idx] == CellKind::Air {
                    self.cell_kind[cell_idx] = CellKind::Fluid;
                }
            }
        }

        for component in 0..=1 {
            let (dx, dy, f, prev_f, d) = if component == 0 {
                (0.0, h2, &mut self.u, &mut self.prev_u, &mut self.du)
            } else {
                (h2, 0.0, &mut self.v, &mut self.prev_v, &mut self.dv)
            };

            for i in 0..self.num_particles {
                let p = self.particle_pos[i];
                let x = f32::clamp(p.x, h, (nx - 1.0) * h);
                let y = f32::clamp(p.y, h, (ny - 1.0) * h);

                let x0 = usize::min(f32::floor((x - dx) * h1) as usize, self.num_cells_x - 2);
                let tx = ((x - dx) - x0 as f32 * h) * h1;
                let x1 = usize::min(x0 + 1, self.num_cells_x - 2);

                let y0 = usize::min(f32::floor((y - dy) * h1) as usize, self.num_cells_y - 2);
                let ty = ((y - dy) - y0 as f32 * h) * h1;
                let y1 = usize::min(y0 + 1, self.num_cells_y - 2);

                let sx = 1.0 - tx;
                let sy = 1.0 - ty;

                let d0 = sx * sy;
                let d1 = tx * sy;
                let d2 = tx * ty;
                let d3 = sx * ty;

                let nr0 = x0 * n + y0;
                let nr1 = x1 * n + y0;
                let nr2 = x1 * n + y1;
                let nr3 = x0 * n + y1;

                if to_grid {
                    let pv = self.particle_vel[i][component];
                    f[nr0] += pv * d0;
                    d[nr0] += d0;
                    f[nr1] += pv * d1;
                    d[nr1] += d1;
                    f[nr2] += pv * d2;
                    d[nr2] += d2;
                    f[nr3] += pv * d3;
                    d[nr3] += d3;
                } else {
                    let offset = if component == 0 { n } else { 1 };

                    #[rustfmt::skip]
                    let valid0 = if self.cell_kind[nr0] != CellKind::Air || self.cell_kind[nr0 - offset] != CellKind::Air { 1.0 } else { 0.0 };
                    #[rustfmt::skip]
                    let valid1 = if self.cell_kind[nr1] != CellKind::Air || self.cell_kind[nr1 - offset] != CellKind::Air { 1.0 } else { 0.0 };
                    #[rustfmt::skip]
                    let valid2 = if self.cell_kind[nr2] != CellKind::Air || self.cell_kind[nr2 - offset] != CellKind::Air { 1.0 } else { 0.0 };
                    #[rustfmt::skip]
                    let valid3 = if self.cell_kind[nr3] != CellKind::Air || self.cell_kind[nr3 - offset] != CellKind::Air { 1.0 } else { 0.0 };

                    let v = self.particle_vel[i][component];
                    let d = valid0 * d0 + valid1 * d1 + valid2 * d2 + valid3 * d3;

                    if d > 0.0 {
                        let pic_v = (valid0 * d0 * f[nr0]
                            + valid1 * d1 * f[nr1]
                            + valid2 * d2 * f[nr2]
                            + valid3 * d3 * f[nr3])
                            / d;
                        let corr = (valid0 * d0 * (f[nr0] - prev_f[nr0])
                            + valid1 * d1 * (f[nr1] - prev_f[nr1])
                            + valid2 * d2 * (f[nr2] - prev_f[nr2])
                            + valid3 * d3 * (f[nr3] - prev_f[nr3]))
                            / d;
                        let flip_v = v + corr;

                        self.particle_vel[i][component] =
                            (1.0 - self.flip_ratio) * pic_v + self.flip_ratio * flip_v;
                    }
                }
            }

            if to_grid {
                for i in 0..f.len() {
                    if d[i] > 0.0 {
                        f[i] /= d[i];
                    }
                }

                // restore solid cells
                for i in 0..self.num_cells_x {
                    for j in 0..self.num_cells_y {
                        let ind = i * n + j;
                        let solid = self.cell_kind[ind] == CellKind::Solid;
                        if solid || (i > 0 && self.cell_kind[(i - 1) * n + j] == CellKind::Solid) {
                            self.u[ind] = self.prev_u[ind];
                        }
                        if solid || (j > 0 && self.cell_kind[i * n + j - 1] == CellKind::Solid) {
                            self.v[ind] = self.prev_v[ind];
                        }
                    }
                }
            }
        }
    }

    fn update_particle_density(&mut self) {
        let n = self.num_cells_y;
        let h = self.h;
        let h1 = self.inv_spacing;
        let h2 = 0.5 * h;
        let d = &mut self.particle_density;

        d.fill(0.0);
        for i in 0..self.num_particles {
            let x = self.particle_pos[i];
            let x = Vec2::clamp(
                x,
                Vec2::splat(h),
                Vec2::new(
                    (self.num_cells_x as f32 - 1.0) * h,
                    (self.num_cells_y as f32 - 1.0) * h,
                ),
            );
            let x0 = Vec2::floor((x - h2) * h1);
            let t = ((x - h2) - x0 * h) * h1;
            let x0 = x0.as_uvec2();
            let x1 = UVec2::min(
                x0 + 1,
                UVec2::new(self.num_cells_x as u32 - 2, self.num_cells_y as u32 - 2),
            );
            let s = 1.0 - t;

            let y0 = x0.y as usize;
            let x0 = x0.x as usize;
            let y1 = x1.y as usize;
            let x1 = x1.x as usize;
            if x0 < self.num_cells_x && y0 < self.num_cells_y {
                d[x0 * n + y0] += s.x * s.y;
            };
            if x1 < self.num_cells_x && y0 < self.num_cells_y {
                d[x1 * n + y0] += t.x * s.y;
            };
            if x1 < self.num_cells_x && y1 < self.num_cells_y {
                d[x1 * n + y1] += t.x * t.y;
            };
            if x0 < self.num_cells_x && y1 < self.num_cells_y {
                d[x0 * n + y1] += s.x * t.y;
            };
        }

        if self.particle_rest_density == 0.0 {
            let mut sum = 0.0;
            let mut num_fluid_cells = 0.0;
            for (i, id) in d.iter().enumerate().take(self.num_cells) {
                if self.cell_kind[i] == CellKind::Fluid {
                    sum += id;
                    num_fluid_cells += 1.0;
                }
            }

            if num_fluid_cells > 0.0 {
                self.particle_rest_density = sum / num_fluid_cells;
            }
        }
    }

    fn solve_incompressibility(&mut self) {
        self.p.fill(0.0);
        self.prev_u.clone_from_slice(&self.u);
        self.prev_v.clone_from_slice(&self.v);

        let n = self.num_cells_y;
        let cp = self.density * self.h / self.dt;

        for _ in 0..self.num_pressure_iters {
            for i in 1..self.num_cells_x - 1 {
                for j in 1..self.num_cells_y - 1 {
                    if self.cell_kind[i * n + j] != CellKind::Fluid {
                        continue;
                    }
                    let center = i * n + j;
                    let left = (i - 1) * n + j;
                    let right = (i + 1) * n + j;
                    let bottom = i * n + j - 1;
                    let top = i * n + j + 1;

                    let sx0 = self.s[left];
                    let sx1 = self.s[right];
                    let sy0 = self.s[bottom];
                    let sy1 = self.s[top];
                    let s = sx0 + sx1 + sy0 + sy1;

                    if s == 0.0 {
                        continue;
                    }

                    let mut div = self.u[right] - self.u[center] + self.v[top] - self.v[center];
                    if self.particle_rest_density > 0.0 && self.compensate_drift {
                        let k = 1.0;
                        let compression =
                            self.particle_density[i * n + j] - self.particle_rest_density;
                        if compression > 0.0 {
                            div -= k * compression;
                        }
                    }

                    let p = -div / s * self.over_relaxation;
                    self.p[center] += cp * p;

                    self.u[center] -= sx0 * p;
                    self.u[right] += sx1 * p;
                    self.v[center] -= sy0 * p;
                    self.v[top] += sy1 * p;
                }
            }
        }
    }

    fn update_particle_colors(&mut self) {
        let h1 = self.inv_spacing;
        for i in 0..self.num_particles {
            let mut s = 0.01;
            self.particle_color[i] = Vec3::clamp(
                self.particle_color[i] + Vec3::new(-s, -s, s),
                Vec3::splat(0.0),
                Vec3::splat(1.0),
            );
            let x = self.particle_pos[i];
            let xi = usize::clamp(f32::floor(x.x * h1) as usize, 1, self.num_cells_x - 1);
            let yi = usize::clamp(f32::floor(x.y * h1) as usize, 1, self.num_cells_y - 1);
            let cell_idx = xi * self.num_cells_y + yi;

            let d0 = self.particle_rest_density;
            if d0 > 0.0 {
                let rel_density = self.particle_density[cell_idx] / d0;
                if rel_density < 0.7 {
                    s = 0.8;
                    self.particle_color[i] = Vec3::new(s, s, 1.0);
                }
            }
        }
    }

    fn update_cell_colors(&mut self) {
        self.cell_color.iter_mut().for_each(|c| *c = Vec3::ZERO);

        for i in 0..self.num_cells {
            if self.cell_kind[i] == CellKind::Solid {
                self.cell_color[i] = Vec3::splat(0.5);
            } else if self.cell_kind[i] == CellKind::Fluid {
                let mut d = self.particle_density[i];
                if self.particle_rest_density > 0.0 {
                    d /= self.particle_rest_density;
                }
                self.cell_color[i] = get_sci_color(d, 0.0, 2.0).into();
            }
        }
    }

    #[allow(clippy::too_many_lines)]
    pub fn draw(&mut self) {
        self.update_particle_colors();
        self.update_cell_colors();

        let gl = &mut self.renderer.context;

        gl.clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        let sim_width = self.width / self.c_scale;

        // draw colored grid
        if self.show_grid {
            gl.use_program(Some(&self.renderer.particle_program));

            // set uniforms
            let point_size = GRID_POINT_SIZE_SCALE * self.h / sim_width * self.width;
            gl.uniform1f(Some(&self.renderer.particle_point_size_uniform), point_size);
            gl.uniform2f(
                Some(&self.renderer.particle_domain_size_uniform),
                sim_width,
                SIM_HEIGHT,
            );
            gl.uniform1i(Some(&self.renderer.particle_mode_draw_disk_uniform), 0);

            // set position buffer
            set_buffers_and_attributes(
                gl,
                &self.renderer.grid_buffer,
                2,
                self.renderer.particle_position_attrib_location,
            );

            // set color buffer
            set_buffers_and_attributes(
                gl,
                &self.renderer.grid_color_buffer,
                3,
                self.renderer.particle_color_attrib_location,
            );
            unsafe {
                // Note that `Float32Array::view` is somewhat dangerous (hence the
                // `unsafe`!). This is creating a raw view into our module's
                // `WebAssembly.Memory` buffer, but if we allocate more pages for ourself
                // (aka do a memory allocation in Rust) it'll cause the buffer to change,
                // causing the `Float32Array` to be invalid.
                //
                // As a result, after `Float32Array::view` we have to be very careful not to
                // do any memory allocations before it's dropped.
                let colors_f32_view = self.cell_color.as_ptr().cast::<f32>(); // &[Vec3] -> *const Vec3 -> *const f32
                let colors_array_buf_view = js_sys::Float32Array::view(std::slice::from_raw_parts(
                    colors_f32_view,
                    self.num_cells * 3,
                ));
                gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &colors_array_buf_view,
                );
            }

            // draw
            gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, self.num_cells as i32);

            // cleanup
            gl.disable_vertex_attrib_array(self.renderer.particle_position_attrib_location);
            gl.disable_vertex_attrib_array(self.renderer.particle_color_attrib_location);
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        }

        // draw water particles
        if self.show_particles {
            gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
            gl.use_program(Some(&self.renderer.particle_program));

            // set uniforms
            let point_size =
                PARTICLE_POINT_SIZE_SCALE * self.particle_radius / sim_width * self.width;
            gl.uniform1f(Some(&self.renderer.particle_point_size_uniform), point_size);
            gl.uniform2f(
                Some(&self.renderer.particle_domain_size_uniform),
                sim_width,
                SIM_HEIGHT,
            );
            gl.uniform1i(Some(&self.renderer.particle_mode_draw_disk_uniform), 1);

            // set position buffer
            set_buffers_and_attributes(
                gl,
                &self.renderer.particle_buffer,
                2,
                self.renderer.particle_position_attrib_location,
            );
            unsafe {
                // See comment above for safety
                let positions_f32_view = self.particle_pos.as_ptr().cast::<f32>(); // &[Vec2] -> *const Vec2 -> *const f32
                let positions_array_buf_view = js_sys::Float32Array::view(
                    std::slice::from_raw_parts(positions_f32_view, self.num_particles * 2),
                );
                gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &positions_array_buf_view,
                );
            }

            // set color buffer
            set_buffers_and_attributes(
                gl,
                &self.renderer.particle_color_buffer,
                3,
                self.renderer.particle_color_attrib_location,
            );
            unsafe {
                // See comment above for safety
                let colors_f32_view = self.particle_color.as_ptr().cast::<f32>(); // &[Vec3] -> *const Vec3 -> *const f32
                let colors_array_buf_view = js_sys::Float32Array::view(std::slice::from_raw_parts(
                    colors_f32_view,
                    self.num_particles * 3,
                ));
                gl.buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &colors_array_buf_view,
                );
            }

            // draw
            gl.draw_arrays(WebGl2RenderingContext::POINTS, 0, self.num_particles as i32);

            // cleanup
            gl.disable_vertex_attrib_array(self.renderer.particle_position_attrib_location);
            gl.disable_vertex_attrib_array(self.renderer.particle_color_attrib_location);
            gl.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        }

        // draw obstacle disk
        if self.show_obstacle {
            gl.clear(WebGl2RenderingContext::DEPTH_BUFFER_BIT);
            gl.use_program(Some(&self.renderer.mesh_program));
            gl.uniform2f(
                Some(&self.renderer.mesh_domain_size_uniform),
                sim_width,
                SIM_HEIGHT,
            );
            gl.uniform3fv_with_f32_array(
                Some(&self.renderer.mesh_color_uniform),
                &OBSTALCE_COLOR.to_array(),
            );
            gl.uniform2fv_with_f32_array(
                Some(&self.renderer.mesh_translation_uniform),
                &self.obstacle_pos.to_array(),
            );
            gl.uniform1f(
                Some(&self.renderer.mesh_scale_uniform),
                self.obstacle_radius + self.particle_radius,
            );

            set_buffers_and_attributes(
                gl,
                &self.renderer.disk_buffer,
                2,
                self.renderer.mesh_position_attrib_location,
            );
            gl.bind_buffer(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                Some(&self.renderer.disk_id_buffer),
            );
            gl.draw_elements_with_i32(
                WebGl2RenderingContext::TRIANGLES,
                (3 * OBSTACLE_DISK_NUM_SEGS) as i32,
                WebGl2RenderingContext::UNSIGNED_SHORT,
                0,
            );

            gl.disable_vertex_attrib_array(self.renderer.mesh_position_attrib_location);
        }
    }

    pub fn step(&mut self) {
        for _ in 0..self.num_substeps {
            self.integrate_particles();
            if self.separate_particles {
                self.push_particles_apart();
            }
            self.handle_particle_collisions();
            self.transfer_velocities(true);
            self.update_particle_density();
            self.solve_incompressibility();
            self.transfer_velocities(false);
        }
    }

    fn set_obstacle(&mut self, pos: Vec2, reset: bool) {
        let mut v = Vec2::ZERO;

        if !reset {
            v = (pos - self.obstacle_pos) / self.dt;
        }

        self.obstacle_pos = pos;
        self.obstacle_vel = v;
        let r = self.obstacle_radius;
        let n = self.num_cells_y;

        for i in 1..self.num_cells_x - 2 {
            for j in 1..self.num_cells_y - 2 {
                self.s[i * n + j] = 1.0;
                let dx = (i as f32 + 0.5) * self.h - pos.x;
                let dy = (j as f32 + 0.5) * self.h - pos.y;

                if dx * dx + dy * dy < r * r {
                    self.s[i * n + j] = 0.0;
                    self.u[i * n + j] = v.x;
                    self.u[(i + 1) * n + j] = v.x;
                    self.v[i * n + j] = v.y;
                    self.v[i * n + (j + 1)] = v.y;
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

impl WebGLRenderer {
    #[allow(clippy::too_many_lines)]
    fn new(
        context: WebGl2RenderingContext,
        width: i32,
        height: i32,
        num_cells_x: usize,
        num_cells_y: usize,
        h: f32,
        num_particles: usize,
    ) -> Result<Self, JsValue> {
        context.viewport(0, 0, width, height);
        context.clear_color(0.0, 0.0, 0.0, 1.0);

        let particle_vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es
        precision mediump float;

        uniform float point_size;
        uniform vec2 domain_size;
        uniform int mode_draw_disk;
        in vec2 position;
        in vec3 color;
        out vec3 frag_color;
        flat out int frag_mode_draw_disk;

        void main() {
            vec4 screen_transform = vec4(2.0 / domain_size.x, 2.0 / domain_size.y, -1.0, -1.0);
            gl_Position = vec4(position * screen_transform.xy + screen_transform.zw, 0.0, 1.0);
            
            gl_PointSize = point_size;
            frag_color = color;
            frag_mode_draw_disk = mode_draw_disk;
        }
        "#,
        )?;
        let particle_frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
        precision mediump float;

        flat in int frag_mode_draw_disk;
        in vec3 frag_color;
        out vec4 out_color;

        void main() {
            if (frag_mode_draw_disk == 1) {
				float rx = 0.5 - gl_PointCoord.x;
				float ry = 0.5 - gl_PointCoord.y;
				float r2 = rx * rx + ry * ry;
				if (r2 > 0.25) {
					discard;
                }
			}
			out_color = vec4(frag_color, 1.0);
        }
        "#,
        )?;
        let particle_program =
            link_program(&context, &particle_vert_shader, &particle_frag_shader)?;

        let mesh_vert_shader = compile_shader(
            &context,
            WebGl2RenderingContext::VERTEX_SHADER,
            r#"#version 300 es
        precision mediump float;

        uniform vec2 domain_size;
        uniform vec3 color;
        uniform vec2 translation;
        uniform float scale;
        in vec2 position;
        out vec3 frag_color;

        void main() {
			vec2 v = translation + position * scale;
		    vec4 screen_transform = vec4(2.0 / domain_size.x, 2.0 / domain_size.y, -1.0, -1.0);
		    gl_Position = vec4(v * screen_transform.xy + screen_transform.zw, 0.0, 1.0);

		    frag_color = color;
        }
        "#,
        )?;
        let mesh_frag_shader = compile_shader(
            &context,
            WebGl2RenderingContext::FRAGMENT_SHADER,
            r#"#version 300 es
        precision mediump float;

        in vec3 frag_color;
        out vec4 out_color;

        void main() {
            out_color = vec4(frag_color, 1.0);
        }
        "#,
        )?;
        let mesh_program = link_program(&context, &mesh_vert_shader, &mesh_frag_shader)?;

        // particle shader uniforms
        let particle_point_size_uniform = context
            .get_uniform_location(&particle_program, "point_size")
            .expect("Unable to get shader point size uniform location");
        let particle_domain_size_uniform = context
            .get_uniform_location(&particle_program, "domain_size")
            .expect("Unable to get shader domain size uniform location");
        let particle_mode_draw_disk_uniform = context
            .get_uniform_location(&particle_program, "mode_draw_disk")
            .expect("Unable to get shader mode draw disk uniform location");

        // preallocate particle position vertex buffer
        context.use_program(Some(&particle_program));
        let particle_position_attrib_location =
            context.get_attrib_location(&particle_program, "position") as u32;
        let particle_buffer = context
            .create_buffer()
            .ok_or("Failed to create particle position buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&particle_buffer));
        let zeroed = vec![0.0; num_particles * 2];
        unsafe {
            let positions_array_buf_view = js_sys::Float32Array::view(&zeroed);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &positions_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        // preallocate particle color buffer
        let particle_color_attrib_location =
            context.get_attrib_location(&particle_program, "color") as u32;
        let particle_color_buffer = context
            .create_buffer()
            .ok_or("Failed to create particle color buffer")?;
        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&particle_color_buffer),
        );
        let zeroed = vec![0.0; num_particles * 3];
        unsafe {
            let colors_array_buf_view = js_sys::Float32Array::view(&zeroed);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &colors_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        // mesh shader uniforms
        let mesh_domain_size_uniform = context
            .get_uniform_location(&mesh_program, "domain_size")
            .expect("Unable to get shader domain size uniform location");
        let mesh_color_uniform = context
            .get_uniform_location(&mesh_program, "color")
            .expect("Unable to get shader color uniform location");
        let mesh_translation_uniform = context
            .get_uniform_location(&mesh_program, "translation")
            .expect("Unable to get shader translation uniform location");
        let mesh_scale_uniform = context
            .get_uniform_location(&mesh_program, "scale")
            .expect("Unable to get shader scale uniform location");

        // prepare disk mesh
        context.use_program(Some(&mesh_program));
        let mesh_position_attrib_location =
            context.get_attrib_location(&mesh_program, "position") as u32;
        let disk_buffer = context
            .create_buffer()
            .ok_or("Failed to create disk buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&disk_buffer));
        let dphi = 2.0 * PI / OBSTACLE_DISK_NUM_SEGS as f32;
        let mut disk_verts: Vec<f32> = Vec::new();
        disk_verts.push(0.0);
        disk_verts.push(0.0);
        for i in 0..OBSTACLE_DISK_NUM_SEGS {
            disk_verts.push(f32::cos(i as f32 * dphi));
            disk_verts.push(f32::sin(i as f32 * dphi));
        }
        unsafe {
            let disk_verts_buf_view = js_sys::Float32Array::view(&disk_verts);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &disk_verts_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, None);
        let disk_id_buffer = context
            .create_buffer()
            .ok_or("Failed to create disk id buffer")?;
        context.bind_buffer(
            WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
            Some(&disk_id_buffer),
        );
        let mut disk_ids: Vec<u16> = Vec::new();
        for i in 0..OBSTACLE_DISK_NUM_SEGS {
            disk_ids.push(0);
            disk_ids.push(1 + i as u16);
            disk_ids.push(1 + (i as u16 + 1) % OBSTACLE_DISK_NUM_SEGS as u16);
        }
        unsafe {
            let disk_ids_buf_view = js_sys::Uint16Array::view(&disk_ids);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER,
                &disk_ids_buf_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }
        context.bind_buffer(WebGl2RenderingContext::ELEMENT_ARRAY_BUFFER, None);

        // prepare grid vertex locations
        context.use_program(Some(&particle_program));
        let grid_buffer = context
            .create_buffer()
            .ok_or("Failed to create grid vertex buffer")?;
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(&grid_buffer));
        let mut cell_centers: Vec<f32> = Vec::new();
        for i in 0..num_cells_x {
            for j in 0..num_cells_y {
                cell_centers.push((i as f32 + 0.5) * h);
                cell_centers.push((j as f32 + 0.5) * h);
            }
        }
        unsafe {
            let grid_buffer_view = js_sys::Float32Array::view(&cell_centers);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &grid_buffer_view,
                WebGl2RenderingContext::STATIC_DRAW,
            );
        }

        // preallocate grid color buffer
        let grid_color_buffer = context
            .create_buffer()
            .ok_or("Failed to create grid color buffer")?;
        context.bind_buffer(
            WebGl2RenderingContext::ARRAY_BUFFER,
            Some(&grid_color_buffer),
        );
        let zeroed = vec![0.0; num_particles * 3];
        unsafe {
            let colors_array_buf_view = js_sys::Float32Array::view(&zeroed);
            context.buffer_data_with_array_buffer_view(
                WebGl2RenderingContext::ARRAY_BUFFER,
                &colors_array_buf_view,
                WebGl2RenderingContext::DYNAMIC_DRAW,
            );
        }

        Ok(Self {
            context,

            particle_program,
            particle_buffer,
            particle_color_buffer,
            grid_buffer,
            grid_color_buffer,
            particle_position_attrib_location,
            particle_color_attrib_location,
            particle_point_size_uniform,
            particle_domain_size_uniform,
            particle_mode_draw_disk_uniform,

            mesh_program,
            disk_buffer,
            disk_id_buffer,
            mesh_position_attrib_location,
            mesh_domain_size_uniform,
            mesh_color_uniform,
            mesh_translation_uniform,
            mesh_scale_uniform,
        })
    }
}
