import { sveltekit } from '@sveltejs/kit/vite'
import { plugins } from './src/config/vite.js'
import examples from 'mdsvexamples/vite'
import sveld from './docs/plugins/sveld.js'
import Generator from './src/plugins/generator/index.js'
import precompileIntl from 'svelte-intl-precompile/sveltekit-plugin'

/** @type {import('vite').UserConfig} */
const config = {
	plugins: [
		sveltekit(),
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
}

export default config
