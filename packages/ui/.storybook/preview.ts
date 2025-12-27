import '../src/styles/tailwind.css'

import type { Preview } from '@storybook/vue3-vite'

const preview: Preview = {
	parameters: {
		controls: {
			matchers: {
				color: /(background|color)$/i,
				date: /Date$/i,
			},
		},
	},
}

export default preview
