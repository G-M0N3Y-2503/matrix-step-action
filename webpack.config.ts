import { Configuration } from 'webpack'
import * as path from 'path'

const config: Configuration = {
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
}

export default config
