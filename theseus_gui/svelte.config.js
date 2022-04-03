import adapter from '@sveltejs/adapter-static';
import path from "path";
import { preprocess, plugins } from 'omorphia/config/svelte.config'

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess,
    kit: {
        adapter: adapter({
            fallback: '200.html',
        }),
        vite: {
            plugins: [
                ...plugins,
            ],
            resolve: {
                alias: {
                    $assets: path.resolve('./src/assets'),
                    $components: path.resolve('./src/components'),
                    $layout: path.resolve('./src/layout'),
                    $lib: path.resolve('./src/lib'),
                    $stores: path.resolve('./src/stores'),
                    $styles: path.resolve('./src/styles'),
                    $generated: path.resolve('./src/generated'),
                },
            },
        }
    }
};

export default config;
