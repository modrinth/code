import preset from '@modrinth/tooling-config/tailwind/tailwind-preset.ts'
import type { Config } from 'tailwindcss'

const config: Config = {
	content: [
		'./src/**/*.{js,vue,ts,mdx}',
		'./.storybook/**/*.{ts,js}',
		'../../apps/frontend/src/**/*.{js,vue,ts,mdx}',
	],
	presets: [preset],
}

export default config
