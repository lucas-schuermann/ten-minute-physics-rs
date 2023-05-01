const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = (env, argv) => {
    console.log('Building in %s mode', argv.mode);

    config = {
        entry: './index.ts',
        resolve: {
            extensions: ['.ts', '.js'],
        },
        module: {
            rules: [
                {
                    test: /\.ts$/,
                    use: 'ts-loader',
                    exclude: /node_modules/,
                },
            ],
        },
        output: {
            path: path.resolve(__dirname, 'dist'),
            filename: './index.js',
        },
        plugins: [
            new HtmlWebpackPlugin({
                template: './index.html'
            }),
            new WasmPackPlugin({
                // See https://github.com/GoogleChromeLabs/wasm-bindgen-rayon/#readme
                // Other compilation flags provided in npm scripts, see `package.json`
                mode: "production",
                extraArgs: `--target web -- . -Z build-std=panic_abort,std`,
                crateDirectory: path.resolve(__dirname, "."),
            })
        ],
        experiments: {
            asyncWebAssembly: true
        },
        performance: {
            // disable hints banner since WASM modules will be large in size
            hints: false
        },
        devServer: {
            // Required in order to use SharedArrayBuffer
            // See https://web.dev/coop-coep/
            headers: {
                'Cross-Origin-Embedder-Policy': 'require-corp',
                'Cross-Origin-Opener-Policy': 'same-origin',
            }
        }
    };
    return config;
}