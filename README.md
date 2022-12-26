![Build](https://github.com/cerrno/ten-minute-physics-rs/actions/workflows/main.yml/badge.svg)

`ten-minute-physics-rs` reimplements Matthias Müller's ["Ten Minute Physics"](https://matthias-research.github.io/pages/tenMinutePhysics/index.html) series in Rust with WASM + WebGL.

For all demos, please see https://cerrno.github.io/ten-minute-physics-rs/. This project is deployed to Github Pages after building with Github Actions.

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