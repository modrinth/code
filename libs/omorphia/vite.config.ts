import { resolve } from 'path'
import { defineConfig } from 'vite'
import svgLoader from 'vite-svg-loader'
import eslintPlugin from 'vite-plugin-eslint'
import vue from '@vitejs/plugin-vue'
import dts from 'vite-plugin-dts'
import nodeExternals from 'rollup-plugin-node-externals'

export default defineConfig({
  build: {
    minify: false,
    lib: {
      entry: resolve(__dirname, 'lib/index.ts'),
      name: 'Omorphia',
      fileName: 'omorphia',
      formats: ['es'],
    },
  },
  plugins: [
    { enforce: 'pre', ...nodeExternals() },
    vue(),
    svgLoader({
      svgoConfig: {
        plugins: [
          {
            name: 'preset-default',
            params: {
              overrides: {
                removeViewBox: false,
              },
            },
          },
        ],
      },
    }),
    eslintPlugin(),
    dts(),
  ],
  resolve: {
    alias: {
      '@': resolve(__dirname, './lib'),
    },
  },
})
