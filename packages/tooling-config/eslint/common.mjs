import simpleImportSort from 'eslint-plugin-simple-import-sort'

export default [
	{
		plugins: {
			'simple-import-sort': simpleImportSort,
		},
		rules: {
			'simple-import-sort/imports': 'error',
			'simple-import-sort/exports': 'error',
		},
	},
	{
		rules: {
			'@typescript-eslint/no-type-alias': 'off',
			'@typescript-eslint/ban-ts-comment': 'off',
			'@typescript-eslint/prefer-literal-enum-member': 'off',
			'@typescript-eslint/no-namespace': 'off',
			'@typescript-eslint/no-invalid-void-type': 'off',
		},
	},
]
