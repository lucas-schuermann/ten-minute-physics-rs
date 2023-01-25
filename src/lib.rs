#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

#[allow(clippy::many_single_char_names)]
fn get_sci_color(val: f32, min: f32, max: f32) -> [f32; 3] {
    let mut val = val.clamp(min, max - 0.0001);
    let d = max - min;
    val = if d == 0.0 { 0.5 } else { (val - min) / d };
    let m = 0.25;
    let num = f32::floor(val / m);
    let s = (val - num * m) / m;
    let (r, g, b) = match num as u8 {
        0 => (0.0, s, 1.0),
        1 => (0.0, 1.0, 1.0 - s),
        2 => (s, 1.0, 0.0),
        3 => (1.0, 1.0 - s, 0.0),
        _ => (1.0, 0.0, 0.0),
    };
    [r, g, b]
}

fn get_sci_color_255(val: f32, min: f32, max: f32) -> [f32; 3] {
    let [r, g, b] = get_sci_color(val, min, max);
    [255.0 * r, 255.0 * g, 255.0 * b]
}

mod cloth_14;
mod flip_18;
mod fluid_sim_17;
mod gpu_cloth_16;
mod hashing_11;
mod mesh;
mod self_collision_15;
mod softbodies_10;
mod softbody_skinning_12;
