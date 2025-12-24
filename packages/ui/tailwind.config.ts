import type { Config } from 'tailwindcss'

import preset from '../../packages/tooling-config/tailwind/tailwind-preset'

const config: Config = {
	content: ['./src/components/**/*.{js,vue,ts}', './src/pages/**/*.{js,vue,ts}'],
	presets: [preset],
}

export default config
