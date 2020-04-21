const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");

module.exports = {
    entry: "./www/bootstrap.js",
    output: {
        filename: "main.js",
        path: path.resolve(__dirname, "dist"),
    },
    mode: "production",
    plugins: [
        new WasmPackPlugin({
            crateDirectory: path.resolve(__dirname, "."),
            outName: "luminous_ld46",
            watchDirectories: [
                path.resolve(__dirname, "engine/src"),
                path.resolve(__dirname, "www"),
            ],
        }),
        new HtmlWebpackPlugin({
            title: "Wispering Away by Luminous for LD46",
            template: "www/index.html",
        }),
    ],
    resolve: {
        modules: ["node_modules", "pkg"],
    },
    devServer: {
        overlay: true,
        watchOptions: {
            poll: true,
        },
    },
    module: {
        rules: [
            {
                test: /\.(png|m4a|mp3)$/,
                use: [
                    {
                        loader: "url-loader",
                        options: {
                            limit: 8000,
                            name: "images/[hash]-[name].[ext]",
                        },
                    },
                ],
            },
        ],
    },
};
