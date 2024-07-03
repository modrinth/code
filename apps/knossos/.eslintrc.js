// @ts-check
/** @type {import("eslint").ESLint.ConfigData} */
module.exports = {
  root: true,
  extends: [
    'eslint:recommended',
    'plugin:@typescript-eslint/recommended',
    'plugin:vue/vue3-recommended',
    '@nuxtjs/eslint-config-typescript',
    'plugin:prettier/recommended', // Integrate Prettier into ESLint
    'prettier', // Disable unnecessary ESLint rules in the presence of Prettier
    'plugin:import/recommended',
    'plugin:import/typescript',
  ],
  rules: {
    'no-console': 'off',
    'vue/no-v-html': 'off',
    'vue/multi-word-component-names': 'off',
    'import/extensions': ['error', 'always', { ignorePackages: true }],
  },
  settings: {
    'import/parsers': {
      '@typescript-eslint/parser': ['.ts', '.tsx'],
    },
    'import/resolver': {
      typescript: true,
    },
  },
}
