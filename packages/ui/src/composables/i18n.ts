import IntlMessageFormat from 'intl-messageformat'
import type { Ref } from 'vue'
import type { CompileError, MessageCompiler, MessageContext } from 'vue-i18n'

import { injectI18n } from '../providers/i18n'
import { injectI18nDebug } from './i18n-debug'

export interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

export type MessageDescriptorMap<K extends string> = Record<K, MessageDescriptor>

export type CrowdinMessages = Record<string, { message?: string; defaultMessage?: string } | string>

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
	translatedName: MessageDescriptor
	numeric?: Intl.RelativeTimeFormatNumeric
	dir?: 'ltr' | 'rtl'
	iso?: string
	file?: string
}

export const LOCALES: LocaleDefinition[] = [
	// Commented out as it's RTL - will enable when we have better RTL support
	// {
	// 	code: 'ar-SA',
	// 	name: 'العربية (السعودية)',
	// 	translatedName: defineMessage({ id: 'locale.ar-SA', defaultMessage: 'Arabic' }),
	// 	dir: 'rtl',
	// },
	{
		code: 'cs-CZ',
		name: 'Čeština',
		translatedName: defineMessage({ id: 'locale.cs-CZ', defaultMessage: 'Czech' }),
	},
	{
		code: 'da-DK',
		name: 'Dansk',
		translatedName: defineMessage({ id: 'locale.da-DK', defaultMessage: 'Danish' }),
	},
	{
		code: 'de-CH',
		name: 'Deutsch (Schweiz)',
		translatedName: defineMessage({ id: 'locale.de-CH', defaultMessage: 'German (Switzerland)' }),
	},
	{
		code: 'de-DE',
		name: 'Deutsch (Deutschland)',
		translatedName: defineMessage({ id: 'locale.de-DE', defaultMessage: 'German (Germany)' }),
	},
	{
		code: 'en-US',
		name: 'English (United States)',
		translatedName: defineMessage({
			id: 'locale.en-US',
			defaultMessage: 'English (United States)',
		}),
	},
	{
		code: 'es-419',
		name: 'Español (Latinoamérica)',
		translatedName: defineMessage({
			id: 'locale.es-419',
			defaultMessage: 'Spanish (Latin America)',
		}),
	},
	{
		code: 'es-ES',
		name: 'Español (España)',
		translatedName: defineMessage({ id: 'locale.es-ES', defaultMessage: 'Spanish (Spain)' }),
	},
	{
		code: 'fi-FI',
		name: 'Suomi',
		translatedName: defineMessage({ id: 'locale.fi-FI', defaultMessage: 'Finnish' }),
	},
	{
		code: 'fil-PH',
		name: 'Filipino',
		translatedName: defineMessage({ id: 'locale.fil-PH', defaultMessage: 'Filipino' }),
	},
	{
		code: 'fr-FR',
		name: 'Français',
		translatedName: defineMessage({ id: 'locale.fr-FR', defaultMessage: 'French' }),
	},
	{
		code: 'he-IL',
		name: 'עברית',
		translatedName: defineMessage({ id: 'locale.he-IL', defaultMessage: 'Hebrew' }),
		dir: 'rtl',
	},
	{
		code: 'hu-HU',
		name: 'Magyar',
		translatedName: defineMessage({ id: 'locale.hu-HU', defaultMessage: 'Hungarian' }),
	},
	{
		code: 'id-ID',
		name: 'Bahasa Indonesia',
		translatedName: defineMessage({ id: 'locale.id-ID', defaultMessage: 'Indonesian' }),
	},
	{
		code: 'it-IT',
		name: 'Italiano',
		translatedName: defineMessage({ id: 'locale.it-IT', defaultMessage: 'Italian' }),
		numeric: 'always',
	},
	{
		code: 'ja-JP',
		name: '日本語',
		translatedName: defineMessage({ id: 'locale.ja-JP', defaultMessage: 'Japanese' }),
	},
	{
		code: 'ko-KR',
		name: '한국어',
		translatedName: defineMessage({ id: 'locale.ko-KR', defaultMessage: 'Korean' }),
	},
	{
		code: 'ms-MY',
		name: 'Bahasa Melayu',
		translatedName: defineMessage({ id: 'locale.ms-MY', defaultMessage: 'Malay' }),
	},
	{
		code: 'nl-NL',
		name: 'Nederlands',
		translatedName: defineMessage({ id: 'locale.nl-NL', defaultMessage: 'Dutch' }),
	},
	{
		code: 'no-NO',
		name: 'Norsk (Bokmål)',
		translatedName: defineMessage({ id: 'locale.no-NO', defaultMessage: 'Norwegian Bokmål' }),
	},
	{
		code: 'pl-PL',
		name: 'Polski',
		translatedName: defineMessage({ id: 'locale.pl-PL', defaultMessage: 'Polish' }),
	},
	{
		code: 'pt-BR',
		name: 'Português (Brasil)',
		translatedName: defineMessage({ id: 'locale.pt-BR', defaultMessage: 'Portuguese (Brazil)' }),
	},
	{
		code: 'pt-PT',
		name: 'Português (Portugal)',
		translatedName: defineMessage({ id: 'locale.pt-PT', defaultMessage: 'Portuguese (Portugal)' }),
	},
	{
		code: 'ro-RO',
		name: 'Română',
		translatedName: defineMessage({ id: 'locale.ro-RO', defaultMessage: 'Romanian' }),
	},
	{
		code: 'ru-RU',
		name: 'Русский',
		translatedName: defineMessage({ id: 'locale.ru-RU', defaultMessage: 'Russian' }),
		numeric: 'always',
	},
	{
		code: 'sr-CS',
		name: 'Srpski (latinica)',
		translatedName: defineMessage({ id: 'locale.sr-CS', defaultMessage: 'Serbian (Latin)' }),
	},
	{
		code: 'sv-SE',
		name: 'Svenska',
		translatedName: defineMessage({ id: 'locale.sv-SE', defaultMessage: 'Swedish' }),
	},
	{
		code: 'th-TH',
		name: 'ไทย',
		translatedName: defineMessage({ id: 'locale.th-TH', defaultMessage: 'Thai' }),
	},
	{
		code: 'tr-TR',
		name: 'Türkçe',
		translatedName: defineMessage({ id: 'locale.tr-TR', defaultMessage: 'Turkish' }),
	},
	{
		code: 'uk-UA',
		name: 'Українська',
		translatedName: defineMessage({ id: 'locale.uk-UA', defaultMessage: 'Ukrainian' }),
	},
	{
		code: 'vi-VN',
		name: 'Tiếng Việt',
		translatedName: defineMessage({ id: 'locale.vi-VN', defaultMessage: 'Vietnamese' }),
	},
	{
		code: 'zh-CN',
		name: '简体中文',
		translatedName: defineMessage({ id: 'locale.zh-CN', defaultMessage: 'Chinese (Simplified)' }),
	},
	{
		code: 'zh-TW',
		name: '繁體中文',
		translatedName: defineMessage({ id: 'locale.zh-TW', defaultMessage: 'Chinese (Traditional)' }),
	},
]

export function transformCrowdinMessages(messages: CrowdinMessages): Record<string, string> {
	const result: Record<string, string> = {}
	for (const [key, value] of Object.entries(messages)) {
		if (typeof value === 'string') {
			result[key] = value
		} else if (typeof value === 'object' && value !== null) {
			const msg = value.message ?? value.defaultMessage
			if (msg) {
				result[key] = msg
			}
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
		// Extract locale code from path like './locales/en-US/index.json', './src/locales/en-US/index.json' or './locales/en-US/meta.json'
		const match = path.match(/\/([^/]+)\/(index|meta)\.json$/)
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
 * Uses the injected I18nContext from the provider.
 */
export function useVIntl(): VIntlFormatters & { locale: Ref<string> } {
	const { t, locale } = injectI18n()
	const debugContext = injectI18nDebug()

	function formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string {
		// Read locale.value to ensure Vue tracks this as a reactive dependency
		// when formatMessage is called during component render
		void locale.value

		const key = descriptor.id
		const translation = t(key, values ?? {})

		let result: string
		if (translation && translation !== key) {
			result = translation as string
		} else {
			// Fallback to defaultMessage if key not found
			const defaultMsg = descriptor.defaultMessage ?? key
			try {
				const formatter = new IntlMessageFormat(defaultMsg, locale.value)
				result = formatter.format(values ?? {}) as string
			} catch {
				result = defaultMsg
			}
		}

		if (debugContext?.enabled.value) {
			debugContext.registry.set(key, {
				key,
				value: result,
				defaultMessage: descriptor.defaultMessage,
				timestamp: Date.now(),
			})
			if (debugContext.keyReveal.value) {
				return `\u300C${key}\u300D`
			}
		}

		return result
	}

	return { formatMessage, locale }
}
