import config from '@modrinth/tooling-config/eslint/nuxt.mjs'

export default config.append([
	{
		ignores: ['src/generated/app-events/*.ts', 'src/generated/app-events/postcard/**'],
	},
])
