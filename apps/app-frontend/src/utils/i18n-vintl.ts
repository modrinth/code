import IntlMessageFormat from 'intl-messageformat'
import { useI18n } from 'vue-i18n'

export interface MessageDescriptor {
	id: string
	defaultMessage?: string
	description?: string
}

export type MessageDescriptorMap<K extends string> = Record<K, MessageDescriptor>

export function defineMessage<T extends MessageDescriptor>(descriptor: T): T {
	return descriptor
}

export function defineMessages<K extends string, T extends MessageDescriptorMap<K>>(
	descriptors: T,
): T {
	return descriptors
}

export interface VIntlFormatters {
	formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string
}

export function useVIntl(): VIntlFormatters & { locale: globalThis.Ref<string> } {
	const { t, locale, messages } = useI18n()

	function formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string {
		const key = descriptor.id
		const localeMessages = messages.value[locale.value] as Record<string, string> | undefined
		const translation = localeMessages?.[key] ?? (t(key, values ?? {}) as string)

		if (translation && translation !== key) {
			try {
				const formatter = new IntlMessageFormat(translation, locale.value)
				return formatter.format(values ?? {}) as string
			} catch {
				return translation
			}
		}

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
