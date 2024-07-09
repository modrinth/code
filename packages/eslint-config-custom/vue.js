const { resolve } = require('node:path')

const project = resolve(process.cwd(), 'tsconfig.json')

/*
 * This is a custom ESLint configuration for use with
 * internal that utilize VueJS.
 *
 * This config extends the Vercel Engineering Style Guide.
 * For more information, see https://github.com/vercel/style-guide
 *
 */

module.exports = {
  extends: [
    '@vercel/style-guide/eslint/browser',
    '@vue/eslint-config-typescript',
    'eslint-config-prettier',
    'eslint-config-turbo',
  ]
  .map(
    require.resolve,
  ),
  parserOptions: {
    ecmaVersion: 'latest',
  },
  settings: {
    'import/resolver': {
      typescript: {
        project,
      },
    },
  },
  ignorePatterns: ['node_modules/', 'dist/', '.eslintrc.js'],

  rules: {
    'import/no-default-export': 'off',
    'vue/multi-word-component-names': 'off',
    camelcase: 'off',
    'no-console': 'off',
    'no-bitwise': 'off',
    'unicorn/filename-case': 'off',
    'comma-dangle': ['error', 'only-multiline'],
    'vue/no-v-html': 'off',
  },
}
