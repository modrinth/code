import { fileURLToPath } from 'node:url'

import type { StorybookConfig } from '@storybook/vue3-vite'
import { mergeConfig } from 'vite'

const config: StorybookConfig = {
	framework: {
		name: '@storybook/vue3-vite',
		options: {
			docgen: false,
		},
	},
	stories: ['../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
	addons: ['@storybook/addon-themes', '@storybook/addon-a11y'],
	viteFinal: async (config) =>
		mergeConfig(config, {
			resolve: {
				alias: {
					'@modrinth/api-client': fileURLToPath(
						new URL('../../api-client/src/index.ts', import.meta.url),
					),
				},
			},
		}),
}
export default config
