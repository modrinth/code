import { mdsvex } from 'mdsvex'
import mdsvexConfig from './mdsvex.config.js'
import adapter from '@sveltejs/adapter-static'
import examples from 'mdsvexamples/vite'
import sveld from './plugins/sveld.js'
import path from 'path'
import { preprocess, plugins } from './src/package/config/svelte.config.js'
import Generator from './src/package/plugins/generator/index.js'
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin'

/** @type {import('@sveltejs/kit').Config} */
const config = {
	extensions: ['.svelte', ...mdsvexConfig.extensions],

	preprocess: [preprocess, mdsvex(mdsvexConfig)],

	kit: {
		adapter: adapter(),
		prerender: {
			default: true,
			onError: 'continue',
		},
		vite: {
			plugins: [
				Generator({
					gameVersions: true,
				}),
				...plugins,
				examples,
				sveld(),
				precompileIntl('locales'),
			],

			resolve: {
				alias: {
					$package: path.resolve('./src/package'),
					$routes: path.resolve('./src/routes'),
					$generated: path.resolve('./generated'),
					omorphia: path.resolve('./src/package'),
				},
			},

			build: {
				rollupOptions: {
					external: '/_app/COMPONENT_API.json',
				},
			},

			server: {
				fs: {
					allow: ['generated'],
				},
			},
		},
		files: {
			lib: 'src/package',
		},
	},
}

export default config
