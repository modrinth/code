import preset from '@modrinth/tooling-config/tailwind/tailwind-preset.ts'
import type { Config } from 'tailwindcss'

const config: Config = {
	content: [
		'./src/components/**/*.{js,vue,ts}',
		'./src/pages/**/*.{js,vue,ts}',
		'./src/stories/**/*.{js,vue,ts,mdx}',
		'./.storybook/**/*.{ts,js}',
	],
	presets: [preset],
}

export default config
