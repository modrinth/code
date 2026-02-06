import type { InjectionKey, Ref } from 'vue'
import { inject, provide } from 'vue'

export interface RegistryEntry {
	key: string
	value: string
	defaultMessage?: string
	timestamp: number
}

export interface I18nDebugContext {
	enabled: Ref<boolean>
	keyReveal: Ref<boolean>
	registry: Map<string, RegistryEntry>
	panelOpen: Ref<boolean>
}

export const I18N_DEBUG_KEY: InjectionKey<I18nDebugContext> = Symbol('i18n-debug')

export function provideI18nDebug(context: I18nDebugContext): void {
	provide(I18N_DEBUG_KEY, context)
}

export function injectI18nDebug(): I18nDebugContext | null {
	return inject(I18N_DEBUG_KEY, null)
}

export function buildCrowdinUrl(key: string, locale: string): string {
	return `https://crowdin.com/translate/modrinth-platform/all/en-${locale}?filter=basic&value=0&search_type=identifier&search=${encodeURIComponent(key)}`
}
