import eslint from '@eslint/js'
import prettierEslint from 'eslint-plugin-prettier/recommended'
import tseslint from 'typescript-eslint'
import common from './common.mjs'

export default tseslint.config(
	eslint.configs.recommended,
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
