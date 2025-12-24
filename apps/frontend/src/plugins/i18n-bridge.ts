import type { I18nContext } from '@modrinth/ui/src/utils/i18n'
import { computed, isRef, ref } from 'vue'

export default defineNuxtPlugin({
	name: 'i18n-bridge',
	setup(nuxtApp) {
		nuxtApp.hook('app:created', (vueApp) => {
			const i18n = vueApp.config.globalProperties.$i18n
			if (!i18n) return

			const localeRef = isRef(i18n.locale) ? i18n.locale : ref(i18n.locale ?? 'en-US')
			const messagesRef = isRef(i18n.messages) ? i18n.messages : ref(i18n.messages ?? {})

			const flatMessages = computed(() => {
				const locale = localeRef.value
				const allMessages = messagesRef.value
				const localeMessages = allMessages[locale] as Record<string, unknown> | undefined
				if (!localeMessages) return {}

				const result: Record<string, string> = {}
				for (const [key, value] of Object.entries(localeMessages)) {
					if (typeof value === 'string') {
						result[key] = value
					}
				}
				return result
			})

			const i18nContext: I18nContext = {
				locale: localeRef,
				t: (key: string, values?: Record<string, unknown>) => {
					try {
						return i18n.t(key, values ?? {})
					} catch {
						return key
					}
				},
				messages: flatMessages,
			}

			vueApp.provide('i18n-context', i18nContext)
		})
	},
})
