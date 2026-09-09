import config from '@modrinth/tooling-config/eslint/nuxt.mjs'

export default config.append([
	{
		ignores: ['src/generated/app-events/*.ts', 'src/generated/app-events/postcard/**'],
	},
	{
		rules: {
			'turbo/no-undeclared-env-vars': ['error', { allowList: ['^DEV$', '^PROD$'] }],
		},
	},
])
