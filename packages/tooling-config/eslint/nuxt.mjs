import { createConfigForNuxt } from '@nuxt/eslint-config/flat'
import { fixupPluginRules } from '@eslint/compat'
import turboPlugin from 'eslint-plugin-turbo'
import simpleImportSort from 'eslint-plugin-simple-import-sort'

export const configurationNuxtToAppend = [
  {
    name: 'turbo',
    plugins: {
      turbo: fixupPluginRules(turboPlugin),
    },
    rules: {
      'turbo/no-undeclared-env-vars': 'error',
    },
  },
  {
    name: 'modrinth',
    rules: {
      'vue/html-self-closing': 'off',
      'vue/multi-word-component-names': 'off',
    },
    languageOptions: {
      parserOptions: {
        warnOnUnsupportedTypeScriptVersion: false,
      },
    },
  },
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
      '@typescript-eslint/ban-ts-comment': 'off',
    },
  },
]

export default createConfigForNuxt().append(configurationNuxtToAppend)
