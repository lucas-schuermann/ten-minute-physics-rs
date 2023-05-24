#![allow(clippy::many_single_char_names, clippy::similar_names)]

use glam::{vec2, Vec2};
use wasm_bindgen::prelude::*;

const DEFAULT_MAX_ITERS: usize = 100;
const DEFAULT_FRACTAL_POS: Vec2 = Vec2::ZERO;
const DEFAULT_FRACTAL_C: Vec2 = vec2(-0.6258, 0.4025);
const FRACTAL_C_STEP_SCALE: f32 = 0.1;
const DEFAULT_SCALE: f32 = 0.003;

const GRADIENT_COLORS: [[f32; 3]; 7] = [
    [15.0, 2.0, 66.0],
    [191.0, 41.0, 12.0],
    [222.0, 99.0, 11.0],
    [229.0, 208.0, 14.0],
    [255.0, 255.0, 255.0],
    [102.0, 173.0, 183.0],
    [14.0, 29.0, 104.0],
];

#[wasm_bindgen(js_name = FractalsSceneType)]
#[derive(PartialEq, Clone, Copy)]
pub enum SceneType {
    Julia,
    Mandelbrot,
}

#[wasm_bindgen]
pub struct FractalsSimulation {
    pub scene_type: SceneType,

    fractal_pos: Vec2,
    fractal_c: Vec2,
    pub max_iters: usize,
    pub scale: f32,

    // rendering
    width: f32,
    height: f32,
    pub draw_mono: bool,
    pub redraw: bool,
}

fn get_gradient_color(val: f32, steps: f32) -> [u8; 3] {
    let num_colors = GRADIENT_COLORS.len() as f32;
    let color0 = f32::floor(val / steps) % num_colors;
    let color1 = (color0 + 1.0) % num_colors;
    let step = val % steps;

    let mut color = [0, 0, 0];

    for i in 0..3 {
        let c0 = GRADIENT_COLORS[color0 as usize][i];
        let c1 = GRADIENT_COLORS[color1 as usize][i];
        color[i] = f32::floor(c0 + (c1 - c0) / steps * step) as u8;
    }
    return color;
}

#[allow(clippy::inline_always)]
#[inline(always)]
fn splat_color(color: &mut [u8; 4], val: u8) {
    color[0..=2].fill(val);
}

#[allow(clippy::inline_always)]
#[inline(always)]
fn set_color(dest: &mut [u8; 4], src: &[u8; 3]) {
    dest[0] = src[0];
    dest[1] = src[1];
    dest[2] = src[2];
}

#[wasm_bindgen]
impl FractalsSimulation {
    #[must_use]
    #[wasm_bindgen(constructor)]
    pub fn new(scene_type: SceneType, width: f32, height: f32) -> FractalsSimulation {
        let width = width.floor();
        let height = height.floor();

        Self {
            scene_type,

            fractal_pos: DEFAULT_FRACTAL_POS,
            fractal_c: DEFAULT_FRACTAL_C,
            max_iters: DEFAULT_MAX_ITERS,
            scale: DEFAULT_SCALE,

            // rendering
            width,
            height,
            draw_mono: false,
            redraw: true,
        }
    }

    pub fn reset(&mut self) {
        self.fractal_pos = DEFAULT_FRACTAL_POS;
        self.fractal_c = DEFAULT_FRACTAL_C;
        self.max_iters = DEFAULT_MAX_ITERS;
        self.scale = DEFAULT_SCALE;

        self.redraw = true;
    }

    fn compute_iters(mut x1: f32, mut x2: f32, c1: f32, c2: f32, max_iters: usize) -> usize {
        for iters in 0..max_iters {
            if x1 * x1 + x2 * x2 > 4.0 {
                return iters;
            }

            let x = x1;
            x1 = x1 * x1 - x2 * x2;
            x2 = 2.0 * x * x2;

            x1 += c1;
            x2 += c2;
        }
        return max_iters;
    }

    pub fn draw_buffer(&mut self, render_buffer: &mut [u8]) {
        if !self.redraw {
            return;
        }
        self.redraw = false;

        let mut color = [255; 4];
        let mut p = 0;

        let mut y = self.fractal_pos.y - self.height / 2.0 * self.scale;
        for _ in 0..=(self.height as usize - 1) {
            let mut x = self.fractal_pos.x - self.width / 2.0 * self.scale;
            for _ in 0..self.width as usize {
                // compute fractal color
                let iters = match self.scene_type {
                    SceneType::Julia => Self::compute_iters(
                        x,
                        y,
                        self.fractal_c.x,
                        self.fractal_c.y,
                        self.max_iters,
                    ),
                    SceneType::Mandelbrot => Self::compute_iters(x, y, x, y, self.max_iters),
                };
                if self.draw_mono {
                    if iters < self.max_iters {
                        splat_color(&mut color, 0);
                    } else {
                        set_color(&mut color, &[255, 192, 0]);
                    }
                } else {
                    if iters < self.max_iters {
                        let grad_color = get_gradient_color(iters as f32, 20.0);
                        set_color(&mut color, &grad_color);
                    } else {
                        splat_color(&mut color, 0);
                    }
                }

                // write color
                render_buffer[p..p + 4].copy_from_slice(&color);
                p += 4;

                x += self.scale;
            }
            y += self.scale;
        }
    }

    pub fn handle_drag(&mut self, dx: f32, dy: f32, alter_c: bool) {
        let d = Vec2::new(dx, dy);
        if alter_c {
            self.fractal_c += FRACTAL_C_STEP_SCALE * d * self.scale;
        } else {
            self.fractal_pos -= d * self.scale;
        }
        self.redraw = true;
    }
}
