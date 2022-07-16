import { mdsvex } from 'mdsvex'
import mdsvexConfig from './mdsvex.config.js'
import adapter from '@sveltejs/adapter-static'
import path from 'path'
import { preprocess } from './src/config/svelte.js'

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
