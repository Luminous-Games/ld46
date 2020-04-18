const path = require("path");
const WasmPackPlugin = require("@wasm-tool/wasm-pack-plugin");
const HtmlWebpackPlugin = require("html-webpack-plugin");

module.exports = {
  entry: "./www/bootstrap.js",
  output: {
    filename: "main.js",
    path: path.resolve(__dirname, "dist"),
  },
  mode: "development",
  plugins: [
    new WasmPackPlugin({
      crateDirectory: path.resolve(__dirname, "."),
      outName: "luminous_ld46",
    }),
    new HtmlWebpackPlugin({
      title: "LD46 game by Luminous",
      template: "www/index.html",
    }),
  ],
  resolve: {
    modules: ["node_modules", "pkg"],
  },
  devServer: {
    watchOptions: {
      poll: true,
    },
  },
};
