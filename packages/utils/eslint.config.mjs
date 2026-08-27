import config from '@modrinth/tooling-config/eslint/nuxt.mjs'
import tseslint from 'typescript-eslint'

export default config.append([
	{
		name: 'modrinth/restrict-comark',
		files: ['**/*.ts'],
		ignores: ['markdown/*.ts'],
		languageOptions: {
			parser: tseslint.parser,
		},
		rules: {
			'no-restricted-imports': [
				'error',
				{
					paths: [
						{
							name: 'comark',
							message: "Use '@modrinth/utils' instead",
						},
					],
					patterns: [
						{
							group: ['comark/*', '@comark/html', '@comark/html/*'],
							message: "Use '@modrinth/utils' instead",
						},
					],
				},
			],
		},
	},
])
