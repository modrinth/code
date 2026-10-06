import { createConfigForNuxt } from '@nuxt/eslint-config/flat'
import common from './common.mjs'

export const configurationNuxtToAppend = [
	...common,
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
						'NuxtRouteAnnouncer',
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
