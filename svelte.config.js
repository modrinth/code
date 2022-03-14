import {mdsvex} from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';
import adapter from '@sveltejs/adapter-auto';
import preprocess from 'svelte-preprocess';
import Icons from 'unplugin-icons/vite';
import svelteSvg from '@poppanator/sveltekit-svg';

/** @type {import('@sveltejs/kit').Config} */
const config = {
    extensions: ['.svelte', ...mdsvexConfig.extensions],

    preprocess: [
        preprocess({
            postcss: true,
        }),
        mdsvex(mdsvexConfig),
    ],

    kit: {
        adapter: adapter(),
        vite: {
            plugins: [
                svelteSvg(),
                Icons({
                    compiler: 'svelte',
                    defaultClass: 'icon',
                }),
            ],
        },
    },
};

export default config;
