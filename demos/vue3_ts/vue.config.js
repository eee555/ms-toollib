const { defineConfig } = require('@vue/cli-service')
module.exports = defineConfig({
  transpileDependencies: true,
  configureWebpack: {
    experiments: {
      asyncWebAssembly: true,  // 启用异步 WebAssembly（推荐）
      // syncWebAssembly: true  // 同步模式（Webpack 5 已弃用）
    },
    module: {
      rules: [
        {
          test: /\.wasm$/,     // 匹配 .wasm 文件
          type: 'webassembly/async'  // 声明模块类型
        }
      ]
    }
  }
})
