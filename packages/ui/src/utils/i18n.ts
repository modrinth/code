import IntlMessageFormat from 'intl-messageformat'
import { inject, type InjectionKey, provide, type Ref, ref } from 'vue'

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

export interface I18nContext {
	locale: Ref<string>
	t: (key: string, values?: Record<string, unknown>) => string
	messages: Ref<Record<string, string>>
}

const i18nKey: InjectionKey<I18nContext> = Symbol('i18n')

export function provideI18n(context: I18nContext) {
	provide(i18nKey, context)
}

export function useI18nContext(): I18nContext {
	const injectedContext = inject(i18nKey)
	if (injectedContext) {
		return injectedContext
	}

	const bridgeContext = inject<I18nContext>('i18n-context', null as unknown as I18nContext)
	if (bridgeContext) {
		return bridgeContext
	}

	return {
		locale: ref('en-US'),
		t: (key: string) => key,
		messages: ref({}),
	}
}

export interface VIntlFormatters {
	formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string
}

export function useVIntl(): VIntlFormatters & { locale: Ref<string> } {
	const { t, locale, messages } = useI18nContext()

	function formatMessage(descriptor: MessageDescriptor, values?: Record<string, unknown>): string {
		const key = descriptor.id
		const translation = messages.value[key] ?? t(key, values)

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
