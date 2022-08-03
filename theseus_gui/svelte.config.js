import adapter from '@sveltejs/adapter-static';
import { preprocess } from 'omorphia/config/svelte';
import path from 'path';

/** @type {import('@sveltejs/kit').Config} */
const config = {
  preprocess: [preprocess],
  kit: {
    adapter: adapter({
      fallback: '200.html'
    }),

    alias: {
      $generated: path.resolve('./generated'),
      $stores: path.resolve('./src/stores'),
      $assets: path.resolve('./src/assets'),
      $components: path.resolve('./src/components'),
      $layout: path.resolve('./src/layout'),
      $lib: path.resolve('./src/lib'),
      $styles: path.resolve('./src/styles')
    }
  }
};

export default config;
