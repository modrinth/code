import path from 'node:path'
import { fileURLToPath } from 'node:url'

import type { StorybookConfig } from '@storybook/vue3-vite'
import { mergeConfig } from 'vite'

const storybookDirectory = path.dirname(fileURLToPath(import.meta.url))

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
					'@modrinth/api-client': path.resolve(storybookDirectory, '../../api-client/src/index.ts'),
				},
			},
		}),
}
export default config
