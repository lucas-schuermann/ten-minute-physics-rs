#![feature(sync_unsafe_cell)]
#![allow(
    clippy::cast_sign_loss,
    clippy::cast_possible_truncation,
    clippy::cast_precision_loss
)]

mod body_chain_challenge;
mod cloth_14;
mod fire_21;
mod flip_18;
mod fluid_2d_challenge;
mod fluid_sim_17;
mod fractals_19;
mod hashing_11;
mod heightfield_water_20;
mod mesh;
mod parallel_cloth_16;
mod self_collision_15;
mod softbodies_10;
mod softbody_skinning_12;

pub mod util {
    use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlProgram, WebGlShader};

    #[must_use]
    #[allow(clippy::many_single_char_names)]
    pub fn get_sci_color(val: f32, min: f32, max: f32) -> [f32; 3] {
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

    #[must_use]
    pub fn get_sci_color_255(val: f32, min: f32, max: f32) -> [f32; 3] {
        let [r, g, b] = get_sci_color(val, min, max);
        [255.0 * r, 255.0 * g, 255.0 * b]
    }

    pub fn set_buffers_and_attributes(
        context: &WebGl2RenderingContext,
        buffer: &WebGlBuffer,
        attrib_size: i32,
        attrib_location: u32,
    ) {
        context.bind_buffer(WebGl2RenderingContext::ARRAY_BUFFER, Some(buffer));
        context.vertex_attrib_pointer_with_i32(
            attrib_location,
            attrib_size,
            WebGl2RenderingContext::FLOAT,
            false,
            0,
            0,
        );
        context.enable_vertex_attrib_array(attrib_location);
    }

    /// # Errors
    /// Will return `Err` if unable to create or compile shader
    pub fn compile_shader(
        context: &WebGl2RenderingContext,
        shader_type: u32,
        source: &str,
    ) -> Result<WebGlShader, String> {
        let shader = context
            .create_shader(shader_type)
            .ok_or_else(|| String::from("Unable to create shader object"))?;
        context.shader_source(&shader, source);
        context.compile_shader(&shader);

        if context
            .get_shader_parameter(&shader, WebGl2RenderingContext::COMPILE_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(shader)
        } else {
            Err(context
                .get_shader_info_log(&shader)
                .unwrap_or_else(|| String::from("Unknown error creating shader")))
        }
    }

    /// # Errors
    /// Will return `Err` if unable to create or link shader program
    pub fn link_program(
        context: &WebGl2RenderingContext,
        vert_shader: &WebGlShader,
        frag_shader: &WebGlShader,
    ) -> Result<WebGlProgram, String> {
        let program = context
            .create_program()
            .ok_or_else(|| String::from("Unable to create program object"))?;

        context.attach_shader(&program, vert_shader);
        context.attach_shader(&program, frag_shader);
        context.link_program(&program);

        if context
            .get_program_parameter(&program, WebGl2RenderingContext::LINK_STATUS)
            .as_bool()
            .unwrap_or(false)
        {
            Ok(program)
        } else {
            Err(context
                .get_program_info_log(&program)
                .unwrap_or_else(|| String::from("Unknown error creating program object")))
        }
    }
}
