use std::{fs, path::Path};

use spirv_builder::{MetadataPrintout, SpirvBuilder};
use spirv_cross::{glsl, spirv};

#[allow(clippy::cast_ptr_alignment)]
pub fn words_from_bytes(buf: &[u8]) -> &[u32] {
    unsafe {
        std::slice::from_raw_parts(
            buf.as_ptr() as *const u32,
            buf.len() / std::mem::size_of::<u32>(),
        )
    }
}

fn main() {
    // tell Cargo that if the given file changes, to rerun this build script
    println!(
        "cargo:rerun-if-changed=src/gpu_cloth_16_shader/src/lib.rs,src/flip_18_shader/src/lib.rs"
    );

    let shaders = ["src/gpu_cloth_16_shader", "src/flip_18_shader"];

    for crate_path in shaders {
        eprintln!("Compiling + disassembling shader {}", crate_path);
        // compile shader from Rust->SPIR-V (similar to LLVM-IR for GPUs)
        let shader_path = SpirvBuilder::new(crate_path, "spirv-unknown-spv1.5")
            .print_metadata(MetadataPrintout::None)
            .build()
            .unwrap()
            .module
            .unwrap_single()
            .to_path_buf();

        // read compiled shader and disassemble to GLSL ES 3.0 (supported by WebGL)
        let shader_source = fs::read(shader_path).unwrap();
        let module = spirv::Module::from_words(words_from_bytes(shader_source.as_slice()));
        let mut ast = spirv::Ast::<glsl::Target>::parse(&module).unwrap();
        let mut options = glsl::CompilerOptions::default();
        options.version = glsl::Version::V3_00Es;

        // write separate output for each shader stage
        for entry_point in &ast.get_entry_points().unwrap() {
            eprintln!("Emitting GLSL shader for entry point {:?}", entry_point);
            let output_name = entry_point.name.clone();
            options.entry_point = Some((output_name.clone(), entry_point.execution_model));
            ast.set_compiler_options(&options).unwrap();
            let output_source = ast.compile().unwrap();
            let output_path = Path::new(crate_path).join(format!("{output_name}.glsl"));
            fs::write(output_path, output_source).unwrap();
        }
    }
}
