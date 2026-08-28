import preset from '@modrinth/tooling-config/tailwind/tailwind-preset.ts'
import type { Config } from 'tailwindcss'

const config: Config = {
	content: [
		'./src/**/*.{js,vue,ts,mdx}',
		'./src/app.vue',
		'../../packages/**/*.{js,vue,ts}',
		'!../../packages/**/node_modules/**',
	],
	presets: [preset],
}

export default config
