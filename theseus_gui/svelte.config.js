import adapter from '@sveltejs/adapter-static';
import { preprocess } from 'omorphia/config/svelte';
import path from 'path';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: [ preprocess ],
    kit: {
        adapter: adapter({
            fallback: '200.html',
        }),

        alias: {
            $generated: path.resolve('./generated'),
            $stores: path.resolve('./src/stores'),
        }
    }
};

export default config;
