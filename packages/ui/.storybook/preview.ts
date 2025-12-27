import '../src/styles/tailwind.css'

import { withThemeByClassName } from '@storybook/addon-themes'
import type { Preview } from '@storybook/vue3-vite'
import { setup } from '@storybook/vue3-vite'
import { createPlugin } from '@vintl/vintl/plugin'

// Set up VIntl for Storybook - provides useVIntl() context for components
const vintlPlugin = createPlugin({
	controllerOpts: {
		defaultLocale: 'en-US',
		locale: 'en-US',
	},
	globalMixin: false,
})

setup((app) => {
	app.use(vintlPlugin)
})

const preview: Preview = {
	parameters: {
		controls: {
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i,
			},
		},
	},
	decorators: [
		withThemeByClassName<Renderer>({
			themes: {
				light: 'light-mode',
				dark: 'dark-mode',
				oled: 'oled-mode',
			},
			defaultTheme: 'dark',
		}),
	],
}

export default preview
