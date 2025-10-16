import config from '@modrinth/tooling-config/eslint/nuxt.mjs'
export default config.append([
	{
		rules: {
			'@typescript-eslint/no-explicit-any': 'off',
			'import/no-unresolved': 'off',
			'no-undef': 'off',
		},
	},
])
