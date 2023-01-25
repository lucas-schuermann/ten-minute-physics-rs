#![cfg_attr(target_arch = "spirv", no_std)]
// HACK(eddyb) can't easily see warnings otherwise from `spirv-builder` builds.
#![deny(warnings)]

use spirv_std::glam::{vec4, Vec2, Vec3, Vec4, Vec4Swizzles};
use spirv_std::spirv;

// // Note: This cfg is incorrect on its surface, it really should be "are we compiling with std", but
// // we tie #[no_std] above to the same condition, so it's fine.
// #[cfg(target_arch = "spirv")]
// use spirv_std::num_traits::Float;
//

#[spirv(vertex)]
pub fn particle_vs(
    #[spirv(position)] out_pos: &mut Vec4,
    #[spirv(point_size)] out_point_size: &mut f32,
    in_position: Vec2,
    in_color: Vec3,
    out_color: &mut Vec3,
    #[spirv(flat)] out_frag_mode_draw_disk: &mut i32,
    #[spirv(uniform, descriptor_set = 1, binding = 0)] point_size: &f32,
    #[spirv(uniform, descriptor_set = 1, binding = 1)] domain_size: &Vec2,
    #[spirv(uniform, descriptor_set = 1, binding = 2)] mode_draw_disk: &i32,
) {
    let screen_transform = vec4(2.0 / domain_size.x, 2.0 / domain_size.y, -1.0, -1.0);
    let out_pos_xy = in_position * screen_transform.xy() + screen_transform.zw();
    *out_pos = vec4(out_pos_xy.x, out_pos_xy.y, 0.0, 1.0);

    *out_point_size = *point_size;
    *out_color = in_color;
    *out_frag_mode_draw_disk = *mode_draw_disk;
}

#[spirv(fragment)]
pub fn particle_fs(
    #[spirv(point_coord)] in_point_coord: Vec2,
    in_color: Vec3,
    #[spirv(flat)] in_mode_draw_disk: i32,
    out_color: &mut Vec4,
) {
    if in_mode_draw_disk == 1 {
        let rx = 0.5 - in_point_coord.x;
        let ry = 0.5 - in_point_coord.y;
        let r2 = rx * rx + ry * ry;
        if r2 > 0.25 {
            return;
        }
    }
    *out_color = in_color.extend(1.0);
}
