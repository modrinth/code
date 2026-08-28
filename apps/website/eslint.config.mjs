import config from '@modrinth/tooling-config/eslint/nuxt.mjs'
export default config.append([
	{
		rules: {
			'import/no-unresolved': 'off',
			'no-undef': 'off',
		},
	},
])
