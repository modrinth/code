import type { Config } from 'tailwindcss'

import preset from '../../packages/tooling-config/tailwind/tailwind-preset'

const config: Config = {
	content: [
		'./src/components/**/*.{js,vue,ts}',
		'./src/layouts/**/*.vue',
		'./src/pages/**/*.vue',
		'./src/plugins/**/*.{js,ts}',
		'./src/App.vue',
		'./src/error.vue',
		// monorepo - TODO: migrate this to its own package
		'../../packages/**/*.{js,vue,ts}',
		'!../../packages/**/node_modules/**',
	],
	presets: [preset],
}

export default config
