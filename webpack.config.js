const path = require('path');
const HtmlWebpackPlugin = require('html-webpack-plugin');
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");

module.exports = (_, argv) => {
    console.log('Building in %s mode', argv.mode);
    config = {
        entry: './crates/demo/index.ts',
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
            filename: './crates/demo/index.js',
        },
        plugins: [
            new HtmlWebpackPlugin({
                template: './crates/demo/index.html'
            }),
            new WasmPackPlugin({
                crateDirectory: path.resolve(__dirname, "./crates/demo")
            })
        ],
        experiments: {
            asyncWebAssembly: true
        },
        performance: {
            // disable hints banner since WASM modules will be large in size
            hints: false
        }
    };
    return config;
}