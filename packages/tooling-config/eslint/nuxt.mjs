import { createConfigForNuxt } from '@nuxt/eslint-config/flat'
import { fixupPluginRules } from '@eslint/compat'
import turboPlugin from 'eslint-plugin-turbo'

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
];

export default createConfigForNuxt().append(configurationNuxtToAppend);