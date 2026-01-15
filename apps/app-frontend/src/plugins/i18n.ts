import { I18N_INJECTION_KEY, type I18nContext } from '@modrinth/ui'
import type { App } from 'vue'

import i18n from '@/i18n.config'

export default {
	install(app: App) {
		// Install vue-i18n as before
		app.use(i18n)

		// Wrap it in our I18nContext interface
		const context: I18nContext = {
			locale: i18n.global.locale,
			t: (key, values) => i18n.global.t(key, values ?? {}) as string,
			setLocale: (newLocale) => {
				i18n.global.locale.value = newLocale
			},
		}

		// Provide the context at app-level
		app.provide(I18N_INJECTION_KEY, context)
	},
}
