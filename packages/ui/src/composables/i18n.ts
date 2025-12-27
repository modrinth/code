import IntlMessageFormat from 'intl-messageformat'
import type { Ref } from 'vue'
import type { CompileError, MessageCompiler, MessageContext } from 'vue-i18n'
import { useI18n } from 'vue-i18n'

export interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

export type MessageDescriptorMap<K extends string> = Record<K, MessageDescriptor>

export type CrowdinMessages = Record<string, { message: string } | string>

export function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return descriptor
}

export function defineMessages<K extends string, T extends MessageDescriptorMap<K>>(
	descriptors: T,
): T {
	return descriptors
}

export interface LocaleDefinition {
	code: string
	name: string
	dir?: 'ltr' | 'rtl'
}

export const LOCALES: LocaleDefinition[] = [
	// { code: 'af-ZA', name: 'Afrikaans' },
	// { code: 'ar-EG', name: 'العربية (مصر)', dir: 'rtl' },
	// { code: 'ar-SA', name: 'العربية (السعودية)', dir: 'rtl' },
	// { code: 'az-AZ', name: 'Azərbaycan' },
	// { code: 'be-BY', name: 'Беларуская' },
	// { code: 'bg-BG', name: 'Български' },
	// { code: 'bn-BD', name: 'বাংলা' },
	// { code: 'ca-ES', name: 'Català' },
	// { code: 'ceb-PH', name: 'Cebuano' },
	// { code: 'cs-CZ', name: 'Čeština' },
	// { code: 'da-DK', name: 'Dansk' },
	{ code: 'de-CH', name: 'Deutsch (Schweiz)' },
	{ code: 'de-DE', name: 'Deutsch' },
	// { code: 'el-GR', name: 'Ελληνικά' },
	// { code: 'en-PT', name: 'Pirate English' },
	// { code: 'en-UD', name: 'Upside Down' },
	{ code: 'en-US', name: 'English (United States)' },
	// { code: 'eo-UY', name: 'Esperanto' },
	{ code: 'es-419', name: 'Español (Latinoamérica)' },
	{ code: 'es-ES', name: 'Español (España)' },
	// { code: 'et-EE', name: 'Eesti' },
	// { code: 'fa-IR', name: 'فارسی', dir: 'rtl' },
	// { code: 'fi-FI', name: 'Suomi' },
	// { code: 'fil-PH', name: 'Filipino' },
	{ code: 'fr-FR', name: 'Français' },
	// { code: 'he-IL', name: 'עברית', dir: 'rtl' },
	// { code: 'hi-IN', name: 'हिन्दी' },
	// { code: 'hr-HR', name: 'Hrvatski' },
	// { code: 'hu-HU', name: 'Magyar' },
	// { code: 'id-ID', name: 'Bahasa Indonesia' },
	// { code: 'is-IS', name: 'Íslenska' },
	{ code: 'it-IT', name: 'Italiano' },
	// { code: 'ja-JP', name: '日本語' },
	// { code: 'kk-KZ', name: 'Қазақша' },
	// { code: 'ko-KR', name: '한국어' },
	// { code: 'ky-KG', name: 'Кыргызча' },
	// { code: 'lol-US', name: 'LOLCAT' },
	// { code: 'lt-LT', name: 'Lietuvių' },
	// { code: 'lv-LV', name: 'Latviešu' },
	// { code: 'ms-Arab', name: 'بهاس ملايو (جاوي)', dir: 'rtl' },
	{ code: 'ms-MY', name: 'Bahasa Melayu' },
	// { code: 'nl-NL', name: 'Nederlands' },
	// { code: 'no-NO', name: 'Norsk' },
	{ code: 'pl-PL', name: 'Polski' },
	{ code: 'pt-BR', name: 'Português (Brasil)' },
	{ code: 'pt-PT', name: 'Português (Portugal)' },
	// { code: 'ro-RO', name: 'Română' },
	{ code: 'ru-RU', name: 'Русский' },
	// { code: 'sk-SK', name: 'Slovenčina' },
	// { code: 'sl-SI', name: 'Slovenščina' },
	// { code: 'sr-CS', name: 'Српски (ћирилица)' },
	// { code: 'sr-SP', name: 'Srpski (latinica)' },
	// { code: 'sv-SE', name: 'Svenska' },
	// { code: 'th-TH', name: 'ไทย' },
	// { code: 'tl-PH', name: 'Tagalog' },
	{ code: 'tr-TR', name: 'Türkçe' },
	// { code: 'tt-RU', name: 'Татарча' },
	{ code: 'uk-UA', name: 'Українська' },
	// { code: 'vi-VN', name: 'Tiếng Việt' },
	{ code: 'zh-CN', name: '简体中文' },
	{ code: 'zh-TW', name: '繁體中文' },
]

export function transformCrowdinMessages(messages: CrowdinMessages): Record<string, string> {
	const result: Record<string, string> = {}
	for (const [key, value] of Object.entries(messages)) {
		if (typeof value === 'string') {
			result[key] = value
		} else if (typeof value === 'object' && value !== null && 'message' in value) {
			result[key] = value.message
		}
	}
	return result
}

const LOCALE_CODES = new Set(LOCALES.map((l) => l.code))

/**
 * Builds locale messages from glob-imported modules.
 * Only includes locales that are defined in the LOCALES array.
 * Usage: buildLocaleMessages(import.meta.glob('./locales/* /index.json', { eager: true }))
 */
export function buildLocaleMessages(
	modules: Record<string, { default: CrowdinMessages }>,
): Record<string, Record<string, string>> {
	const messages: Record<string, Record<string, string>> = {}
	for (const [path, module] of Object.entries(modules)) {
		// Extract locale code from path like './locales/en-US/index.json'
		const match = path.match(/\/([^/]+)\/index\.json$/)
		if (match) {
			const locale = match[1]
			// Only include locales that are in our LOCALES list
			if (LOCALE_CODES.has(locale)) {
				messages[locale] = transformCrowdinMessages(module.default)
			}
		}
	}
	return messages
}

/**
 * Creates a vue-i18n message compiler that uses IntlMessageFormat for ICU syntax support.
 * This enables pluralization, select, and other ICU message features.
 */
export function createMessageCompiler(): MessageCompiler {
	return (msg, { locale, key, onError }) => {
		let messageString: string

		if (typeof msg === 'string') {
			messageString = msg
		} else if (typeof msg === 'object' && msg !== null && 'message' in msg) {
			messageString = (msg as { message: string }).message
		} else {
			onError?.(new Error('Invalid message format') as CompileError)
			return () => key
		}

		try {
			const formatter = new IntlMessageFormat(messageString, locale)
			return (ctx: MessageContext) => {
				try {
					return formatter.format(ctx.values as Record<string, unknown>) as string
				} catch {
					return messageString
				}
			}
		} catch (e) {
			onError?.(e as CompileError)
			return () => key
		}
	}
}
export interface VIntlFormatters {
	formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string
}

/**
 * Composable that provides formatMessage() with the same API as @vintl/vintl.
 * Uses vue-i18n's useI18n() under the hood.
 */
export function useVIntl(): VIntlFormatters & { locale: Ref<string> } {
	const { t, locale } = useI18n()

	function formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string {
		const key = descriptor.id
		const translation = t(key, values ?? {})

		if (translation && translation !== key) {
			return translation as string
		}

		// Fallback to defaultMessage if key not found
		const defaultMsg = descriptor.defaultMessage ?? key
		try {
			const formatter = new IntlMessageFormat(defaultMsg, locale.value)
			return formatter.format(values ?? {}) as string
		} catch {
			return defaultMsg
		}
	}

	return { formatMessage, locale }
}
