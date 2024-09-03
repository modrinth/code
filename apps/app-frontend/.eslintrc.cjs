module.exports = {
  root: true,
  extends: ['../../packages/eslint-config-custom/vue.js'],
  env: {
    browser: true,
    node: true,
  },
  parser: '@typescript-eslint/parser',
  plugins: ['@typescript-eslint'],
  parserOptions: {
    ecmaVersion: 'latest',
    sourceType: 'module',
    project: './tsconfig.json',
  },
  overrides: [
    {
      files: ['*.ts', '*.tsx', '*.vue'],
      rules: {
        '@typescript-eslint/no-unused-vars': ['error'],
      },
    },
  ],
}
