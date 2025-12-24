import { transformCrowdinMessages } from '../i18n.config'

type CrowdinMessages = Record<string, { message: string } | string>

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('../locales/*/index.json', {
	eager: false,
})

export default defineNuxtPlugin({
	name: 'locale-loader',
	async setup(nuxtApp) {
		const i18n = nuxtApp.$i18n

		async function loadLocaleMessages(locale: string): Promise<void> {
			if (locale === 'en-US') return

			const path = `../locales/${locale}/index.json`
			const loader = localeModules[path]

			if (!loader) {
				console.warn(`Locale file not found for: ${locale}`)
				return
			}

			try {
				const messages = await loader()
				const transformed = transformCrowdinMessages(messages.default)
				i18n.setLocaleMessage(locale, transformed)
			} catch (error) {
				console.error(`Failed to load locale: ${locale}`, error)
			}
		}

		const currentLocale = i18n.locale.value
		if (currentLocale !== 'en-US') {
			const messages = i18n.getLocaleMessage(currentLocale)
			if (!messages || Object.keys(messages).length === 0) {
				await loadLocaleMessages(currentLocale)
			}
		}

		nuxtApp.hook('i18n:beforeLocaleSwitch', async ({ newLocale }) => {
			const messages = i18n.getLocaleMessage(newLocale)
			if (!messages || Object.keys(messages).length === 0) {
				await loadLocaleMessages(newLocale)
			}
		})
	},
})
