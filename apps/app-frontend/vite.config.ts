import { resolve } from 'path'
import { defineConfig } from 'vite'
import svgLoader from 'vite-svg-loader'

import vue from '@vitejs/plugin-vue'

const projectRootDir = resolve(__dirname)

// https://vitejs.dev/config/
export default defineConfig({
  resolve: {
    alias: [
      {
        find: '@',
        replacement: resolve(projectRootDir, 'src'),
      },
    ],
  },
  plugins: [
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
  ],

  // Vite options tailored for Tauri development and only applied in `tauri dev` or `tauri build`
  // prevent vite from obscuring rust errors
  clearScreen: false,
  // tauri expects a fixed port, fail if that port is not available
  server: {
    port: 1420,
    strictPort: true,
  },
  // to make use of `TAURI_ENV_DEBUG` and other env variables
  // https://v2.tauri.app/reference/environment-variables/#tauri-cli-hook-commands
  envPrefix: ['VITE_', 'TAURI_'],
  build: {
    // Tauri supports es2021
    target: process.env.TAURI_PLATFORM == 'windows' ? 'chrome105' : 'safari13', // eslint-disable-line turbo/no-undeclared-env-vars
    // don't minify for debug builds
    minify: !process.env.TAURI_DEBUG ? 'esbuild' : false, // eslint-disable-line turbo/no-undeclared-env-vars
    // produce sourcemaps for debug builds
    sourcemap: !!process.env.TAURI_DEBUG, // eslint-disable-line turbo/no-undeclared-env-vars
    commonjsOptions: {
      esmExternals: true,
    },
  },
})
