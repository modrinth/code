import {
	buildLocaleMessages,
	createMessageCompiler,
	type CrowdinMessages,
	LOCALES,
} from '@modrinth/ui'
import englishUi from '@modrinth/ui/src/locales/en-US/index.json'
import { createI18n } from 'vue-i18n'

import englishApp from './locales/en-US/index.json'

const localeUrls = import.meta.glob<string>('./locales/*/index.json', {
	eager: true,
	query: '?url',
	import: 'default',
})
const uiLocaleUrls = import.meta.glob<string>('../../../packages/ui/src/locales/*/index.json', {
	eager: true,
	query: '?url',
	import: 'default',
})

const i18n = createI18n({
	legacy: false,
	locale: 'en-US',
	fallbackLocale: 'en-US',
	messageCompiler: createMessageCompiler(),
	missingWarn: false,
	fallbackWarn: false,
	messages: buildLocaleMessages({
		'./app/en-US/index.json': { default: englishApp },
		'./ui/en-US/index.json': { default: englishUi },
	}),
})

const pendingLocales = new Map<string, Promise<Record<string, string>>>()
let localeRequest = 0

async function fetchMessages(url: string): Promise<CrowdinMessages> {
	const response = await fetch(url)
	if (!response.ok) throw new Error(`Could not load translations: ${response.status}`)
	return response.json()
}

export async function setLocale(requestedLocale: string): Promise<void> {
	const request = ++localeRequest
	const locale = LOCALES.some((candidate) => candidate.code === requestedLocale)
		? requestedLocale
		: 'en-US'
	if (locale !== 'en-US' && locale !== i18n.global.locale.value) {
		let pending = pendingLocales.get(locale)
		if (!pending) {
			pending = Promise.all([
				fetchMessages(localeUrls[`./locales/${locale}/index.json`]),
				fetchMessages(uiLocaleUrls[`../../../packages/ui/src/locales/${locale}/index.json`]),
			])
				.then(
					([app, ui]) =>
						buildLocaleMessages({
							[`./app/${locale}/index.json`]: { default: app },
							[`./ui/${locale}/index.json`]: { default: ui },
						})[locale],
				)
				.finally(() => pendingLocales.delete(locale))
			pendingLocales.set(locale, pending)
		}
		const messages = await pending
		if (request !== localeRequest) return
		i18n.global.setLocaleMessage(locale, messages)
	}
	if (request !== localeRequest) return
	const previous = i18n.global.locale.value
	i18n.global.locale.value = locale
	if (previous !== locale && previous !== 'en-US') {
		i18n.global.setLocaleMessage(previous, {})
	}
}

export default i18n
