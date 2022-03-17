import {mdsvex} from 'mdsvex';
import mdsvexConfig from './mdsvex.config.js';
import adapter from '@sveltejs/adapter-auto';
import preprocess from 'svelte-preprocess';
import sveltePreprocess from 'svelte-preprocess';
import Icons from 'unplugin-icons/vite';
import svelteSvg from '@poppanator/sveltekit-svg';
import {parse} from 'sveltedoc-parser'
import * as svelte from 'svelte/compiler'

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
                {
                    name: "sveltedoc-parser",
                    async transform(src, id) {
                        const query = id.split('?')[1]

                        if ((query || '').includes('raw&api')) {
                            const raw = JSON.parse(src.split('export default ')[1])

                            let {code} = await svelte.preprocess(raw, sveltePreprocess(), {
                                filename: id
                            })

                            const data = await parse({
                                fileContent: code,
                                encoding: 'ascii',
                                features: ['data', 'computed', 'events', 'slots'],
                                ignoredVisibilities: ['private'],
                                includeSourceLocations: true,
                                version: 3
                            })

                            return {
                                code: `export default ${JSON.stringify(data)}`,
                                map: null
                            }
                        }
                    }
                },
            ],
        },
    },
};

export default config;
