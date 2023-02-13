//! Adapted from my separate position-based fluid [repository](https://github.com/cerrno/pbd-fluid-rs/)
//! using the `solver` crate and demo [setup](https://github.com/cerrno/pbd-fluid-rs/blob/master/src/lib.rs)

use glam::vec2;
use wasm_bindgen::prelude::*;
use web_sys::{WebGl2RenderingContext, WebGlBuffer, WebGlUniformLocation};

const BLOCK_PARTICLES: usize = 400;
const MAX_PARTICLES: usize = pbd_fluid_solver::MAX_PARTICLES;
const POINT_SIZE: f32 = 3.0;
const DRAW_SCALE: f32 = 250.0;

use crate::util::{compile_shader, link_program, set_buffers_and_attributes};

#[wasm_bindgen]
pub struct PositionBasedFluidSimulation {
    state: pbd_fluid_solver::State,
    renderer: WebGLRenderer,
}

struct WebGLRenderer {
    context: WebGl2RenderingContext,

    boundary_buffer: WebGlBuffer,
    particle_buffer: WebGlBuffer,
    draw_mode_single_color_uniform: WebGlUniformLocation,
    draw_mode_boundary_uniform: WebGlUniformLocation,
    position_attrib_location: u32,
}

#[wasm_bindgen]
impl PositionBasedFluidSimulation {
    /// # Errors
    /// Will return `Err` if unable to initialize webgl2 context and compile/link shader programs.
    #[wasm_bindgen(constructor)]
    pub fn new(
        context: WebGl2RenderingContext,
        width: f32,
        height: f32,
        use_dark_colors: bool,
        dam_particles_x: usize,
        dam_particles_y: usize,
    ) -> Result<PositionBasedFluidSimulation, JsValue> {
        let x_extent = width * 0.5 / DRAW_SCALE;
        let mut state = pbd_fluid_solver::State::new(x_extent);
        state.init_dam_break(dam_particles_x, dam_particles_y);
        let renderer = init_webgl(
            context,
            width as i32,
            height as i32,
            &state.get_boundaries(),
            use_dark_colors,
        )?;
        Ok(Self { state, renderer })
    }

    #[wasm_bindgen(setter)]
    pub fn set_draw_single_color(&self, enabled: bool) {
        self.renderer.context.uniform1i(
            Some(&self.renderer.draw_mode_single_color_uniform),
            enabled.into(),
        );
    }

    #[must_use]
    #[wasm_bindgen(getter)]
    pub fn num_particles(&self) -> usize {
        self.state.num_particles
    }

    #[wasm_bindgen(setter)]
    pub fn set_viscosity(&mut self, viscosity: f32) {
        self.state.viscosity = viscosity;
    }

    #[wasm_bindgen(setter)]
    pub fn set_solver_substeps(&mut self, num_substeps: usize) {
        self.state.set_solver_substeps(num_substeps);
    }

    pub fn step(&mut self) {
        self.state.update();
    }

    pub fn add_block(&mut self) {
        self.state.init_block(BLOCK_PARTICLES);
    }

    pub fn reset(&mut self, dam_particles_x: usize, dam_particles_y: usize) {
        self.state.clear();
        self.state.init_dam_break(dam_particles_x, dam_particles_y);
    }

    pub fn draw(&self) {
        self.renderer
            .context
            .clear(WebGl2RenderingContext::COLOR_BUFFER_BIT);

        // draw boundaries
        set_buffers_and_attributes(
            &self.renderer.context,
            &self.renderer.boundary_buffer,
            2,
            self.renderer.position_attrib_location,
        );
        self.renderer
            .context
            .uniform1i(Some(&self.renderer.draw_mode_boundary_uniform), 1);
        self.renderer
            .context
            .draw_arrays(WebGl2RenderingContext::TRIANGLES, 0, 12);

        // draw particles
        set_buffers_and_attributes(
            &self.renderer.context,
            &self.renderer.particle_buffer,
            2,
            self.renderer.position_attrib_location,
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
            let positions_f32_view = self.state.get_positions().as_ptr().cast::<f32>(); // &[Vec2] -> *const Vec2 -> *const f32
            let positions_array_buf_view = js_sys::Float32Array::view(std::slice::from_raw_parts(
                positions_f32_view,
                self.state.num_particles * 2,
            ));

            self.renderer
                .context
                .buffer_sub_data_with_i32_and_array_buffer_view(
                    WebGl2RenderingContext::ARRAY_BUFFER,
                    0,
                    &positions_array_buf_view,
                );
        }
        self.renderer
            .context
            .uniform1i(Some(&self.renderer.draw_mode_boundary_uniform), 0);
        #[allow(clippy::cast_possible_wrap)]
        self.renderer.context.draw_arrays(
            WebGl2RenderingContext::POINTS,
            0,
            self.state.num_particles as i32,
        );
    }
}

#[allow(clippy::too_many_lines)]
fn init_webgl(
    context: WebGl2RenderingContext,
    width: i32,
    height: i32,
    boundaries: &[[f32; 4]],
    use_dark_colors: bool,
) -> Result<WebGLRenderer, JsValue> {
    context.viewport(0, 0, width, height);
    if use_dark_colors {
        context.clear_color(0.1, 0.1, 0.1, 1.0);
    } else {
        context.clear_color(0.9, 0.9, 0.9, 1.0);
    }

    let vert_shader = compile_shader(
        &context,
        WebGl2RenderingContext::VERTEX_SHADER,
        format!(
            r##"#version 300 es
        precision mediump float;
        const vec4 particle_color_1 = vec4(0.2549019608, 0.4117647059, 1.0, 1.0); // #4169E1
        const vec4 particle_color_2 = vec4(1.0, 0.2549019608, 0.2980392157, 1.0); // #E1414C

        uniform mat4 u_projection_matrix;
        uniform mat4 u_view_matrix;
        uniform int u_draw_mode_single_color;
        in vec2 in_position;
        out vec4 frag_color;

        void main() {{
            gl_PointSize = {POINT_SIZE:.1};
            gl_Position = u_projection_matrix * u_view_matrix * vec4(in_position, 0.0, 1.0);
            if (u_draw_mode_single_color == 1 || int(floor(float(gl_VertexID) / 1000.0)) % 2 == 0) {{
                frag_color = particle_color_1;
            }} else {{
                frag_color = particle_color_2;
            }}
        }}
        "##,
        )
        .as_str(),
    )?;

    let frag_shader = compile_shader(
        &context,
        WebGl2RenderingContext::FRAGMENT_SHADER,
        r#"#version 300 es
        precision mediump float;
        const vec4 boundary_color = vec4(0.4392156863, 0.5019607843, 0.5647058824, 1.0); // #708090

        uniform int u_draw_mode_boundary;
        in vec4 frag_color;
        out vec4 out_color;

        void main() {
            if (u_draw_mode_boundary == 1) {
                out_color = boundary_color;
            } else {
                out_color = frag_color;
            }
        }
        "#,
    )?;
    let program = link_program(&context, &vert_shader, &frag_shader)?;
    context.use_program(Some(&program));

    // set shader matrix uniforms
    let projection_uniform = context
        .get_uniform_location(&program, "u_projection_matrix")
        .expect("Unable to get shader projection matrix uniform location");
    let ortho_matrix = cgmath::ortho(0.0, width as f32, height as f32, 0.0, -1.0, 1.0);
    let ortho_matrix_flattened_ref: &[f32; 16] = ortho_matrix.as_ref();
    context.uniform_matrix4fv_with_f32_array(
        Some(&projection_uniform),
        false,
        ortho_matrix_flattened_ref,
    );
    let view_uniform = context
        .get_uniform_location(&program, "u_view_matrix")
        .expect("Unable to get shader view matrix uniform location");
    let draw_orig = vec2(width as f32 / 2.0, height as f32);
    let view_matrix: [f32; 16] = [
        DRAW_SCALE,
        0.0,
        0.0,
        0.0,
        0.0,
        -DRAW_SCALE, // flip y coordinate from solver
        0.0,
        0.0,
        0.0,
        0.0,
        DRAW_SCALE,
        0.0,
        draw_orig.x,
        draw_orig.y,
        0.0,
        1.0,
    ];
    context.uniform_matrix4fv_with_f32_array(Some(&view_uniform), false, &view_matrix);
    let draw_mode_single_color_uniform = context
        .get_uniform_location(&program, "u_draw_mode_single_color")
        .expect("Unable to get vertex color mode uniform location");
    context.uniform1i(Some(&draw_mode_single_color_uniform), 0);
    let draw_mode_boundary_uniform = context
        .get_uniform_location(&program, "u_draw_mode_boundary")
        .expect("Unable to get fragment boundary uniform location");
    let position_attrib_location = context.get_attrib_location(&program, "in_position") as u32;

    // prepopulate boundary geometry
    let boundaries: Vec<f32> = boundaries
        .iter()
        .flat_map(|p| {
            // specified as [x0, x0+width, y0, y0+height]
            let x = p[0];
            let y = p[2];
            let w = p[1] - p[0];
            let h = p[3] - p[2];
            // form a rectangle using two triangles, three vertices each
            [
                [x, y],
                [x + w, y],
                [x + w, y + h],
                [x, y],
                [x, y + h],
                [x + w, y + h],
            ]
        })
        .flatten()
        .collect();
    let boundary_buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    set_buffers_and_attributes(&context, &boundary_buffer, 2, position_attrib_location);
    unsafe {
        let boundaries_array_buf_view = js_sys::Float32Array::view(&boundaries);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &boundaries_array_buf_view,
            WebGl2RenderingContext::STATIC_DRAW,
        );
    }

    // preallocate particle vertex buffer
    let particle_buffer = context.create_buffer().ok_or("Failed to create buffer")?;
    set_buffers_and_attributes(&context, &particle_buffer, 2, position_attrib_location);
    let zeroed = vec![0.0; MAX_PARTICLES * 2];
    unsafe {
        let positions_array_buf_view = js_sys::Float32Array::view(&zeroed);

        context.buffer_data_with_array_buffer_view(
            WebGl2RenderingContext::ARRAY_BUFFER,
            &positions_array_buf_view,
            WebGl2RenderingContext::DYNAMIC_DRAW,
        );
    }

    Ok(WebGLRenderer {
        context,

        boundary_buffer,
        particle_buffer,
        draw_mode_single_color_uniform,
        draw_mode_boundary_uniform,
        position_attrib_location,
    })
}
