import { mdsvex } from 'mdsvex'
import mdsvexConfig from './mdsvex.config.js'
import adapter from '@sveltejs/adapter-static'
import examples from 'mdsvexamples/vite'
import sveld from './docs/plugins/sveld.js'
import path from 'path'
import { preprocess, plugins } from './src/config/svelte.config.js'
import Generator from './src/plugins/generator/index.js'
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
					$generated: path.resolve('./generated'),
					omorphia: path.resolve('./src'),
				},
			},

			build: {
				rollupOptions: {
					external: '/_app/COMPONENT_API.json',
				},
			},

			server: {
				fs: {
					allow: ['generated', 'docs'],
				},
			},
		},
		files: {
			assets: 'docs/static',
			hooks: 'docs/hooks',
			lib: 'src',
			params: 'docs/params',
			routes: 'docs/routes',
			serviceWorker: 'docs/service-worker',
			template: 'docs/app.html',
		},
	},
}

export default config
