import baseConfig from '@modrinth/tooling-config/eslint/nuxt.mjs'
import storybook from 'eslint-plugin-storybook'

export default baseConfig.append([
	{
		name: 'storybook',
		files: ['**/*.stories.@(js|jsx|ts|tsx)', '**/.storybook/**/*.@(js|ts)'],
		plugins: {
			storybook,
		},
		rules: {
			...storybook.configs.recommended.rules,
		},
	},
])
