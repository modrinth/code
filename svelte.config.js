import { mdsvex } from 'mdsvex'
import mdsvexConfig from './mdsvex.config.js'
import adapter from '@sveltejs/adapter-static'
import examples from 'mdsvexamples/vite'
import sveld from './docs/plugins/sveld.js'
import path from 'path'
import { preprocess, plugins } from './src/config/svelte.js'
import Generator from './src/plugins/generator/index.js'
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin'

/** @type {import('@sveltejs/kit').Config} */
export default {
	extensions: ['.svelte', ...mdsvexConfig.extensions],

	preprocess: [preprocess, mdsvex(mdsvexConfig)],

	kit: {
		adapter: adapter(),
		prerender: {
			default: true,
			onError: 'continue',
		},

		alias: {
			$generated: path.resolve('./generated'),
			omorphia: path.resolve('./src'),
			['$stores/account']: path.resolve('./docs/dummyStore.ts'),
		},

		vite: {
			plugins: [
				Generator({
					gameVersions: true,
					openapi: true,
				}),
				...plugins,
				examples,
				sveld(),
				precompileIntl('locales'),
			],

			build: {
				rollupOptions: {
					external: ['/_app/COMPONENT_API.json'],
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
