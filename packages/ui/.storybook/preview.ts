import '@modrinth/assets/omorphia.scss'
import 'floating-vue/dist/style.css'
import '../src/styles/tailwind.css'

import { withThemeByClassName } from '@storybook/addon-themes'
import type { Preview } from '@storybook/vue3-vite'
import { setup } from '@storybook/vue3-vite'
import FloatingVue from 'floating-vue'
import { createI18n } from 'vue-i18n'

import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
} from '../src/composables/i18n'

// Load locale messages from the UI package's locales
// @ts-ignore
const localeModules = import.meta.glob('../src/locales/*/index.json', {
	eager: true,
}) as Record<string, { default: CrowdinMessages }>

// Set up vue-i18n for Storybook - provides useVIntl() context for components
const i18n = createI18n({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages(localeModules),
})

setup((app) => {
	app.use(i18n)
	app.use(FloatingVue, {
		themes: {
			'ribbit-popout': {
				$extend: 'dropdown',
				placement: 'bottom-end',
				instantMove: true,
				distance: 8,
			},
			'dismissable-prompt': {
				$extend: 'dropdown',
				placement: 'bottom-start',
			},
		},
	})

	// Create teleport target for components that use <Teleport to="#teleports">
	if (typeof document !== 'undefined' && !document.getElementById('teleports')) {
		const teleportTarget = document.createElement('div')
		teleportTarget.id = 'teleports'
		document.body.appendChild(teleportTarget)
	}
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
		withThemeByClassName({
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
