import { setMarkdownUserCountryResolver } from '@modrinth/utils'
import type { Ref } from 'vue'
import { hasInjectionContext } from 'vue'

import { createContext } from './create-context'

const [injectUserCountryContext, provideUserCountryContext] = createContext<Ref<string>>(
	'root',
	'userCountry',
)

let clientCountry: Ref<string> | null = null

export const injectUserCountry = injectUserCountryContext
export const useUserCountry = injectUserCountryContext

export function provideUserCountry(country: Ref<string>) {
	if (typeof window !== 'undefined') {
		clientCountry = country
	}

	return provideUserCountryContext(country)
}

setMarkdownUserCountryResolver(() => {
	const injectedCountry = hasInjectionContext() ? injectUserCountryContext(null) : null
	return injectedCountry?.value ?? clientCountry?.value
})
