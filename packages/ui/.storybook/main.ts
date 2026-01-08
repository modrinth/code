import type { StorybookConfig } from '@storybook/vue3-vite'

const config: StorybookConfig = {
	framework: '@storybook/vue3-vite',
	core: {
		builder: '@storybook/builder-vite',
	},
	stories: ['../src/**/*.mdx', '../src/**/*.stories.@(js|jsx|mjs|ts|tsx)'],
	addons: [
		'@storybook/addon-themes',
		'@storybook/addon-vitest',
		'@storybook/addon-a11y',
		'@storybook/addon-docs',
		'@storybook/addon-onboarding',
	],
}
export default config
