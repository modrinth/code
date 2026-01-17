import {
	type CrowdinMessages,
	I18N_INJECTION_KEY,
	type I18nContext,
	LOCALES,
	transformCrowdinMessages,
} from '@modrinth/ui'
import IntlMessageFormat from 'intl-messageformat'
import { LRUCache } from 'lru-cache'

const DEFAULT_LOCALE = 'en-US'

const localeModules = import.meta.glob<{ default: CrowdinMessages }>('../locales/*/index.json', {
	eager: false,
})

const messageCache = new LRUCache<string, Record<string, string>>({ max: 10 })
const formatterCache = new LRUCache<string, IntlMessageFormat>({ max: 1000 })
const loadingPromises = new Map<string, Promise<void>>() // Dedupe concurrent loads

async function loadLocale(code: string): Promise<void> {
	if (messageCache.has(code)) return

	// Dedupe concurrent requests for the same locale
	const existing = loadingPromises.get(code)
	if (existing) return existing

	const promise = (async () => {
		const loader = localeModules[`../locales/${code}/index.json`]
		if (!loader) return
		const raw = await loader()
		messageCache.set(code, transformCrowdinMessages(raw.default))
	})()

	loadingPromises.set(code, promise)
	try {
		await promise
	} finally {
		loadingPromises.delete(code)
	}
}

function parseAcceptLanguage(header: string): string | null {
	try {
		for (const lang of header
			.split(',')
			.map((l) => l.split(';')[0]?.trim())
			.filter(Boolean)) {
			const exact = LOCALES.find((loc) => loc.code === lang)
			if (exact) return exact.code
			const prefix = LOCALES.find((loc) => loc.code.startsWith(lang.split('-')[0] + '-'))
			if (prefix) return prefix.code
		}
	} catch {
		// Malformed header, ignore
	}
	return null
}

export default defineNuxtPlugin({
	name: 'i18n',
	enforce: 'pre',
	async setup(nuxtApp) {
		const locale = useState<string>('i18n-locale', () => DEFAULT_LOCALE)

		function t(key: string, values?: Record<string, unknown>): string {
			const currentLocale = locale.value
			const msg = messageCache.get(currentLocale)?.[key] ?? messageCache.get(DEFAULT_LOCALE)?.[key]
			if (!msg) return key
			if (!values || Object.keys(values).length === 0) return msg

			const cacheKey = `${currentLocale}:${msg}`
			let formatter = formatterCache.get(cacheKey)
			if (!formatter) {
				formatter = new IntlMessageFormat(msg, currentLocale)
				formatterCache.set(cacheKey, formatter)
			}
			try {
				const result = formatter.format(values) as string
				if (import.meta.dev && typeof result !== 'string') {
					console.error('[i18n] t() returned non-string:', typeof result)
				}
				return result
			} catch {
				return msg
			}
		}

		async function setLocale(newLocale: string): Promise<void> {
			if (!LOCALES.some((l) => l.code === newLocale)) return
			await loadLocale(newLocale)
			locale.value = newLocale
			useCookie('locale', { maxAge: 31536000, path: '/' }).value = newLocale
		}

		// Detect initial locale (cookie > Accept-Language > default)
		const cookieLocale = useCookie('locale').value
		let detectedLocale = DEFAULT_LOCALE
		if (cookieLocale && LOCALES.some((l) => l.code === cookieLocale)) {
			detectedLocale = cookieLocale
		} else if (import.meta.server) {
			const acceptLang = useRequestHeaders(['accept-language'])['accept-language']
			if (acceptLang) {
				detectedLocale = parseAcceptLanguage(acceptLang) ?? DEFAULT_LOCALE
			}
		}

		// Load locales (hits cache after first request)
		await loadLocale(DEFAULT_LOCALE)
		if (detectedLocale !== DEFAULT_LOCALE) await loadLocale(detectedLocale)
		locale.value = detectedLocale

		const context: I18nContext = { locale, t, setLocale }
		nuxtApp.vueApp.provide(I18N_INJECTION_KEY, context)
	},
})
