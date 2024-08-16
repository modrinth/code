const { resolve } = require('node:path')

const project = resolve(process.cwd(), 'tsconfig.json')

module.exports = {
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended',
    '@nuxtjs/eslint-config-typescript',
    'plugin:prettier/recommended',
    'prettier',
    'plugin:import/recommended',
    'plugin:import/typescript',
    'turbo',
  ],
  parserOptions: {
    sourceType: 'module',
  },
  settings: {
    'import/resolver': {
      typescript: {
        project,
      },
    },
  },
  ignorePatterns: ['.nuxt/**', '.output/**', 'node_modules', 'dist/**'],
  rules: {
    'no-console': 'off',
    'vue/no-v-html': 'off',
    'vue/multi-word-component-names': 'off',
    'vue/no-multiple-template-root': 'off',
    'import/extensions': ['error', 'always', { ignorePackages: true }],
  },
}
