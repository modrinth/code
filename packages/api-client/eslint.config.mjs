import config from '@modrinth/tooling-config/eslint/nuxt.mjs'
import nPlugin from 'eslint-plugin-n'

export default config.append([
	{
		ignores: ['dist/'],
	},
	{
		plugins: {
			n: nPlugin,
		},
		settings: {
			n: {
				tryExtensions: ['.js', '.ts', '.mjs', '.mts', '.cjs', '.cts', '.json', '.node'],
				typescriptExtensionMap: [['.ts', '.js']],
			},
		},
		rules: {
			'n/file-extension-in-import': ['error', 'always'],
		},
	},
])
