import eslint from '@eslint/js'
import prettierEslint from 'eslint-plugin-prettier/recommended'
import tseslint from 'typescript-eslint'

export default tseslint.config(
	eslint.configs.recommended,
	tseslint.configs.recommendedTypeChecked,
	prettierEslint,
	...common,
	{
		languageOptions: {
			parserOptions: {
				warnOnUnsupportedTypeScriptVersion: false,
			},
		},
	},
	{
		ignores: ['node_modules/', 'dist/', 'build/'],
	},
)
