import * as webpack from 'webpack'
import * as path from 'path'

const config: webpack.Configuration = {
  mode: 'production',
  target: 'node',
  experiments: {
    asyncWebAssembly: true,
  },
  entry: './src/index.ts',
  output: {
    path: path.resolve(__dirname, 'dist'),
    filename: '[name].js',
    webassemblyModuleFilename: '[id].wasm',
    clean: true,
  },
  devtool: 'source-map',
  module: {
    rules: [
      {
        test: /\.js$/,
        enforce: 'pre',
        use: ['source-map-loader'],
      },
    ],
  },
}

export default config
