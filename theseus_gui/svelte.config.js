import adapter from '@sveltejs/adapter-static';
import preprocess from 'svelte-preprocess';
import path from "path";
import Icons from 'unplugin-icons/vite';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    preprocess: preprocess({
        postcss: true,
    }),
    kit: {
        adapter: adapter({
            fallback: '200.html',
        }),
        vite: {
            plugins: [
                Icons({
                    compiler: 'svelte',
                }),
            ],
            resolve: {
                alias: {
                    $assets: path.resolve('./src/assets'),
                    $components: path.resolve('./src/components'),
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
