import eslint from '@eslint/js';
import tseslint from 'typescript-eslint';
import prettierEslint from "eslint-plugin-prettier/recommended"

export default tseslint.config(
  eslint.configs.recommended,
  ...tseslint.configs.recommended,
  prettierEslint,
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
);