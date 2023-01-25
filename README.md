![Build](https://github.com/cerrno/ten-minute-physics-rs/actions/workflows/main.yml/badge.svg)

`ten-minute-physics-rs` reimplements Matthias Müller's ["Ten Minute Physics"](https://matthias-research.github.io/pages/tenMinutePhysics/index.html) demos in Rust with WASM + WebGL. Compared with the source pure Javascript implementations, many Rust versions are *~3x faster*.

For all demos, please see https://cerrno.github.io/ten-minute-physics-rs/. Most can be interacted with by dragging the camera and/or objects on screen. This project is deployed to Github Pages after building with Github Actions.

Files in the `src/` directory are labeled according to the corresponding source demo number. The top level `index.{html,ts}` files contain boilerplate code for all demos (such as initializing the `<canvas>` element, [stats.js](https://github.com/mrdoob/stats.js/), and [lil-gui](https://github.com/georgealways/lil-gui)). Each `{demo}.ts` file implements the generic interface defined in `{lib}.ts`, defining scene initialization parameters ([THREE.js](https://github.com/mrdoob/three.js/) or `canvas`), scene setup (generally creating rendering elements, binding GUI elements, etc.), and update/render functions called by the `index.ts` main loop. Each `{demo}.rs` file implements a physics simulation in Rust, which is compiled to WASM using [wasm-pack](https://github.com/rustwasm/wasm-pack) and instantiated/called by the corresponding `{demo}.ts` file to step the simulation.

Memory is shared between the WASM instance and JS via [WebAssembly.Memory](https://developer.mozilla.org/en-US/docs/WebAssembly/JavaScript_interface/Memory) and Rust helper methods which return pointers to contiguous memory locations. For example, particle positions are stored in Rust as a `Vec<Vec3>` struct field. A `wasm-bindgen` [getter](https://rustwasm.github.io/wasm-bindgen/reference/attributes/on-js-imports/getter-and-setter.html) is defined to return a `*const Vec3`. Each `glam::Vec3` is `repr(C)`, so the positions `Vec` is a linear array of `f32`s somewhere in the `WebAssembly.Memory` buffer. For rendering in JS, a `THREE.BufferAttribute` is defined to reference a `Float32Array` referencing the WASM memory `ArrayBuffer` with the pointer (byte offset) returned from the Rust getter and a known length (in this case, `sim.num_particles * 3`). 

## Running
```bash
# install dependencies
npm install

# compile to WASM, run webpack, and spawn a local server
npm run serve
```
Then visit http://localhost:8080

## License
This project is distributed under the [MIT license](LICENSE.md).