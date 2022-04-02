import {mdsvex} from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';
import adapter from '@sveltejs/adapter-static';
import preprocess from 'svelte-preprocess';
import sveltePreprocess from 'svelte-preprocess';
import Icons from 'unplugin-icons/vite';
import svelteSvg from '@poppanator/sveltekit-svg';
import examples from 'mdsvexamples/vite'
import sveld from './plugins/sveld.js'
import path from "path";

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
        prerender: {
            default: true,
            onError: 'continue',
        },
        vite: {
            plugins: [
                svelteSvg(),
                Icons({
                    compiler: 'svelte',
                    defaultClass: 'icon',
                    scale: 1.1428, // 1.1428rem = 16px when root size is 14px
                }),
                examples,
                sveld(),
            ],

            resolve: {
                alias: {
                    $package: path.resolve('./src/package'),
                    $routes: path.resolve('./src/routes'),
                    omorphia: path.resolve('./src/package'),
                },
            },

            build: {
                rollupOptions: {
                    external: '/_app/COMPONENT_API.json'
                }
            }
        },
    },
};

export default config;
