import '@modrinth/assets/omorphia.scss'
import 'floating-vue/dist/style.css'
import '../src/styles/tailwind.css'

import { withThemeByClassName } from '@storybook/addon-themes'
import type { Preview } from '@storybook/vue3-vite'
import { setup } from '@storybook/vue3-vite'
import FloatingVue from 'floating-vue'
import { defineComponent, ref } from 'vue'
import { createI18n } from 'vue-i18n'

import NotificationPanel from '../src/components/nav/NotificationPanel.vue'
import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
} from '../src/composables/i18n'
import {
	AbstractWebNotificationManager,
	type NotificationPanelLocation,
	provideNotificationManager,
	type WebNotification,
} from '../src/providers'

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

class StorybookNotificationManager extends AbstractWebNotificationManager {
	private readonly state = ref<WebNotification[]>([])
	private readonly locationState = ref<NotificationPanelLocation>('right')

	public getNotificationLocation(): NotificationPanelLocation {
		return this.locationState.value
	}

	public setNotificationLocation(location: NotificationPanelLocation): void {
		this.locationState.value = location
	}

	public getNotifications(): WebNotification[] {
		return this.state.value
	}

	protected addNotificationToStorage(notification: WebNotification): void {
		this.state.value.push(notification)
	}

	protected removeNotificationFromStorage(id: string | number): void {
		const index = this.state.value.findIndex((n) => n.id === id)
		if (index > -1) {
			this.state.value.splice(index, 1)
		}
	}

	protected removeNotificationFromStorageByIndex(index: number): void {
		this.state.value.splice(index, 1)
	}

	protected clearAllNotificationsFromStorage(): void {
		this.state.value.splice(0)
	}
}

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

// Wrapper component that provides notification manager context
const NotificationManagerProvider = defineComponent({
	setup(_, { slots }) {
		provideNotificationManager(new StorybookNotificationManager())
		return () => slots.default?.()
	},
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
		// Wrap stories with notification manager provider
		(story) => ({
			components: { story, NotificationManagerProvider, NotificationPanel },
			template: /*html*/ `
				<NotificationManagerProvider>
					<NotificationPanel />
					<story />
				</NotificationManagerProvider>
			`,
		}),
	],
}

export default preview
