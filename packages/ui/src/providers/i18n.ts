import type { InjectionKey, Ref } from 'vue'
import { inject, provide } from 'vue'

// This doesn't use the architecture outlined in index.ts as it needs some custom checks + use the symbol
export interface I18nContext {
	locale: Ref<string>
	t: (key: string, values?: Record<string, unknown>) => string
	setLocale: (locale: string) => Promise<void> | void
}

export const I18N_INJECTION_KEY: InjectionKey<I18nContext> = Symbol('i18n')

export function injectI18n(): I18nContext {
	const context = inject(I18N_INJECTION_KEY)
	if (!context) {
		throw new Error('Injection `Symbol(i18n)` not found. Ensure the i18n plugin is installed.')
	}
	return context
}

export function provideI18n(context: I18nContext): I18nContext {
	provide(I18N_INJECTION_KEY, context)
	return context
}
