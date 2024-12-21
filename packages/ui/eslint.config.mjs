import { createConfigForNuxt } from '@nuxt/eslint-config/flat'
import { fixupPluginRules } from '@eslint/compat'
import turboPlugin from 'eslint-plugin-turbo'

export default createConfigForNuxt().append([
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
  },
])
