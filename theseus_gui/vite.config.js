import { sveltekit } from '@sveltejs/kit/vite';
import path from 'path';
import { plugins } from 'omorphia/config/vite';
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin';
import { Generator } from 'omorphia/plugins';

/** @type {import('vite').UserConfig} */
const config = {
  plugins: [
    sveltekit(),
    ...plugins,
    precompileIntl('locales'),
    Generator({
      gameVersions: true,
      openapi: true
    })
  ],
  optimizeDeps: {
    include: ['highlight.js/lib/core']
  },
  server: {
    fs: {
      allow: ['generated']
    }
  }
};

export default config;
