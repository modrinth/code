import 'floating-vue/dist/style.css'
import '../../assets/styles/defaults.scss'
// frontend css imports
// import '../../../apps/frontend/src/assets/styles/global.scss'
// import '../../../apps/frontend/src/assets/styles/tailwind.css'
// ---
// app-frontend css imports
import '../../../apps/app-frontend/src/assets/stylesheets/global.scss'

import type { Labrinth } from '@modrinth/api-client'
import { GenericModrinthClient } from '@modrinth/api-client'
import { withThemeByClassName } from '@storybook/addon-themes'
import type { Preview } from '@storybook/vue3-vite'
import { setup } from '@storybook/vue3-vite'
import { QueryClient, VueQueryPlugin } from '@tanstack/vue-query'
import FloatingVue from 'floating-vue'
import { computed, defineComponent, h, ref } from 'vue'
import { createI18n } from 'vue-i18n'
import { createMemoryHistory, createRouter } from 'vue-router'

import NotificationPanel from '../src/components/nav/NotificationPanel.vue'
import PopupNotificationPanel from '../src/components/nav/PopupNotificationPanel.vue'
import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
} from '../src/composables/i18n'
import {
	AbstractPopupNotificationManager,
	AbstractWebNotificationManager,
	I18N_INJECTION_KEY,
	type I18nContext,
	type NotificationPanelLocation,
	type PopupNotification,
	provideFilePicker,
	provideModrinthClient,
	provideNotificationManager,
	providePopupNotificationManager,
	provideTags,
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

class StorybookPopupNotificationManager extends AbstractPopupNotificationManager {
	private readonly state = ref<PopupNotification[]>([])

	public getNotifications(): PopupNotification[] {
		return this.state.value
	}

	protected addNotificationToStorage(notification: PopupNotification): void {
		this.state.value.push(notification)
	}

	protected removeNotificationFromStorage(id: string | number): void {
		const index = this.state.value.findIndex((n) => n.id === id)
		if (index > -1) {
			this.state.value.splice(index, 1)
		}
	}

	protected clearAllNotificationsFromStorage(): void {
		this.state.value.splice(0)
	}
}

const StorybookLink = defineComponent({
	name: 'StorybookLink',
	inheritAttrs: false,
	props: {
		to: {
			type: [String, Object],
			default: '',
		},
	},
	setup(props, { attrs, slots }) {
		const href = computed(() => {
			if (typeof props.to === 'string') return props.to || '#'
			if (props.to && typeof props.to === 'object' && 'path' in props.to) {
				const path = props.to.path
				return typeof path === 'string' ? path : '#'
			}
			return '#'
		})

		return () =>
			h(
				'a',
				{
					...attrs,
					href: href.value,
				},
				slots.default?.(),
			)
	},
})

const StorybookClientOnly = defineComponent({
	name: 'StorybookClientOnly',
	setup(_, { slots }) {
		return () => slots.default?.()
	},
})

setup((app) => {
	const queryClient = new QueryClient({
		defaultOptions: {
			queries: {
				retry: false,
			},
			mutations: {
				retry: false,
			},
		},
	})
	app.use(VueQueryPlugin, { queryClient })
	app.use(i18n)

	const router = createRouter({
		history: createMemoryHistory(),
		routes: [{ path: '/:pathMatch(.*)*', component: { render: () => null } }],
	})
	app.use(router)

	app.component('NuxtLink', StorybookLink)
	app.component('RouterLink', StorybookLink)
	app.component('ClientOnly', StorybookClientOnly)

	// Provide the custom I18nContext for components using injectI18n()
	const i18nContext: I18nContext = {
		locale: i18n.global.locale,
		t: (key, values) => i18n.global.t(key, values ?? {}) as string,
		setLocale: (newLocale) => {
			i18n.global.locale.value = newLocale
		},
	}
	app.provide(I18N_INJECTION_KEY, i18nContext)

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

const StorybookProvider = defineComponent({
	setup(_, { slots }) {
		provideNotificationManager(new StorybookNotificationManager())
		providePopupNotificationManager(new StorybookPopupNotificationManager())

		const modrinthClient = new GenericModrinthClient({
			userAgent: 'modrinth-storybook/1.0.0',
		})
		provideModrinthClient(modrinthClient)

		const gameVersions = ref<Labrinth.Tags.v2.GameVersion[]>([])
		const loaders = ref<Labrinth.Tags.v2.Loader[]>([])
		modrinthClient.labrinth.state.build().then((state) => {
			gameVersions.value = state.gameVersions
			loaders.value = state.loaders
		})
		provideTags({ gameVersions, loaders })

		provideFilePicker({
			async pickImage() {
				return null
			},
			async pickModpackFile() {
				return null
			},
		})

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
		(story) => ({
			components: { story, StorybookProvider, NotificationPanel, PopupNotificationPanel },
			template: /*html*/ `
				<StorybookProvider>
					<NotificationPanel />
					<PopupNotificationPanel />
					<story />
				</StorybookProvider>
			`,
		}),
	],
}

export default preview
