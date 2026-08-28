import { layoutsLocaleModules } from '@modrinth/layouts'
import {
	type CrowdinMessages,
	I18N_INJECTION_KEY,
	type I18nContext,
	LOCALES,
	transformCrowdinMessages,
	uiLocaleModules,
	useDebugLogger,
} from '@modrinth/ui'
import IntlMessageFormat from 'intl-messageformat'
import { LRUCache } from 'lru-cache'

const debug = useDebugLogger('i18n')
const DEFAULT_LOCALE = 'en-US'

const websiteLocaleModules = import.meta.glob<{ default: CrowdinMessages }>(
	'../locales/*/index.json',
	{ eager: false },
)

const messageCache = new LRUCache<string, Record<string, string>>({ max: 10 })
const formatterCache = new LRUCache<string, IntlMessageFormat>({ max: 1000 })
const loadingPromises = new Map<string, Promise<void>>()

type LocaleModules = Record<string, () => Promise<{ default: CrowdinMessages }>>

function findLocaleLoader(modules: LocaleModules, code: string) {
	for (const [path, loader] of Object.entries(modules)) {
		if (path.endsWith(`/${code}/index.json`)) {
			return loader
		}
	}
	return undefined
}

function formatIcuMessage(msg: string, locale: string, values: Record<string, unknown>) {
	const cacheKey = `${locale}:${msg}`
	let formatter = formatterCache.get(cacheKey)

	try {
		if (!formatter) {
			formatter = new IntlMessageFormat(msg, locale)
			formatterCache.set(cacheKey, formatter)
		}
		const result = formatter.format(values)
		if (import.meta.dev && typeof result !== 'string') {
			debug('formatIcuMessage: format returned non-string', typeof result)
		}
		return result as string
	} catch {
		return null
	}
}

async function loadLocale(code: string): Promise<void> {
	if (messageCache.has(code)) {
		debug('loadLocale: already cached', code)
		return
	}

	const existing = loadingPromises.get(code)
	if (existing) {
		debug('loadLocale: already loading', code)
		return existing
	}

	debug('loadLocale: starting', code)

	const promise = (async () => {
		const websiteLoader = findLocaleLoader(websiteLocaleModules, code)
		const uiLoader = findLocaleLoader(uiLocaleModules, code)
		const layoutsLoader = findLocaleLoader(layoutsLocaleModules, code)

		debug('loadLocale: loaders found', {
			code,
			website: !!websiteLoader,
			ui: !!uiLoader,
			layouts: !!layoutsLoader,
		})

		const [uiData, layoutsData, websiteData] = await Promise.all([
			uiLoader?.().catch((e) => {
				debug('loadLocale: ui loader failed', code, e)
				return null
			}),
			layoutsLoader?.().catch((e) => {
				debug('loadLocale: layouts loader failed', code, e)
				return null
			}),
			websiteLoader?.().catch((e) => {
				debug('loadLocale: website loader failed', code, e)
				return null
			}),
		])

		debug('loadLocale: data loaded', {
			code,
			uiKeys: uiData ? Object.keys(uiData.default).length : 0,
			layoutsKeys: layoutsData ? Object.keys(layoutsData.default).length : 0,
			websiteKeys: websiteData ? Object.keys(websiteData.default).length : 0,
		})

		const mergedMessages: Record<string, string> = {}
		if (uiData) Object.assign(mergedMessages, transformCrowdinMessages(uiData.default))
		if (layoutsData) Object.assign(mergedMessages, transformCrowdinMessages(layoutsData.default))
		if (websiteData) Object.assign(mergedMessages, transformCrowdinMessages(websiteData.default))

		debug('loadLocale: merged', code, 'total keys:', Object.keys(mergedMessages).length)

		if (Object.keys(mergedMessages).length > 0) {
			messageCache.set(code, mergedMessages)
		}
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
		const localeDir = computed(
			() => LOCALES.find((entry) => entry.code === locale.value)?.dir ?? 'ltr',
		)

		function t(key: string, values?: Record<string, unknown>): string {
			const currentLocale = locale.value
			const localeMessages = messageCache.get(currentLocale)
			const fallbackMessages = messageCache.get(DEFAULT_LOCALE)
			const msg = localeMessages?.[key] ?? fallbackMessages?.[key]

			if (!msg) {
				debug('t: key not found', {
					key,
					locale: currentLocale,
					hasLocaleMessages: !!localeMessages,
					hasFallbackMessages: !!fallbackMessages,
				})
				return key
			}

			if (!values || Object.keys(values).length === 0) return msg

			const formatted = formatIcuMessage(msg, currentLocale, values)
			if (formatted !== null) return formatted

			const fallbackMsg = fallbackMessages?.[key]
			if (fallbackMsg && fallbackMsg !== msg) {
				return formatIcuMessage(fallbackMsg, DEFAULT_LOCALE, values) ?? fallbackMsg
			}

			return msg
		}

		async function setLocale(
			newLocale: string,
			{ persist = true }: { persist?: boolean } = {},
		): Promise<void> {
			debug('setLocale: called', { newLocale, currentLocale: locale.value })

			if (!LOCALES.some((l) => l.code === newLocale)) {
				debug('setLocale: invalid locale', newLocale)
				return
			}

			await loadLocale(newLocale)

			debug('setLocale: loaded', {
				newLocale,
				cacheHas: messageCache.has(newLocale),
				cacheKeys: messageCache.get(newLocale)
					? Object.keys(messageCache.get(newLocale)!).length
					: 0,
			})

			locale.value = newLocale
			if (persist) {
				useCookie('locale', { maxAge: 31536000, path: '/' }).value = newLocale
			}
		}

		const cookieLocale = useCookie('locale').value
		let detectedLocale = DEFAULT_LOCALE
		if (cookieLocale && LOCALES.some((l) => l.code === cookieLocale)) {
			detectedLocale = cookieLocale
			debug('init: locale from cookie', detectedLocale)
		} else if (import.meta.server) {
			const acceptLang = useRequestHeaders(['accept-language'])['accept-language']
			if (acceptLang) {
				detectedLocale = parseAcceptLanguage(acceptLang) ?? DEFAULT_LOCALE
				debug('init: locale from Accept-Language', detectedLocale)
			}
		}

		debug('init: detected locale', { detectedLocale, cookieLocale, isServer: import.meta.server })

		await loadLocale(DEFAULT_LOCALE)
		if (detectedLocale !== DEFAULT_LOCALE) await loadLocale(detectedLocale)
		locale.value = detectedLocale

		debug('init: complete', { locale: locale.value })

		useHead({
			htmlAttrs: {
				lang: locale,
				dir: localeDir,
			},
		})

		const context: I18nContext = { locale, t, setLocale }
		nuxtApp.vueApp.provide(I18N_INJECTION_KEY, context)
	},
})
