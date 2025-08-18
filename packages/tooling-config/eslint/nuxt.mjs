import { fixupPluginRules } from '@eslint/compat'
import { createConfigForNuxt } from '@nuxt/eslint-config/flat'
import turboPlugin from 'eslint-plugin-turbo'
import common from './common.mjs'

export const configurationNuxtToAppend = [
	...common,
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
			'vue/no-undef-components': [
				'error',
				{
					ignorePatterns: [
						'NuxtPage',
						'NuxtLayout',
						'NuxtLink',
						'ClientOnly',
						'Teleport',
						'Transition',
						'TransitionGroup',
						'Head',
						'Title',
						'router-link',
						'RouterView',
						'RouterLink',
						'nuxt-link',
					],
				},
			],
			'vue/no-undef-properties': 'warn',
		},
		languageOptions: {
			parserOptions: {
				warnOnUnsupportedTypeScriptVersion: false,
			},
		},
	},
]

export default createConfigForNuxt().append(configurationNuxtToAppend)
