import { type MaybeRefOrGetter, toValue } from 'vue'

import {
	type FaviconVariant,
	getFaviconHeadLinks,
	resolveFaviconEnvironment,
} from '~/utils/favicon.ts'

export function useFaviconEnvironment() {
	const config = useRuntimeConfig()
	const url = useRequestURL()

	return computed(() => resolveFaviconEnvironment(config, url.hostname))
}

export function useFavicon(variant: MaybeRefOrGetter<FaviconVariant> = 'default') {
	const environment = useFaviconEnvironment()

	useHead({
		link: () => getFaviconHeadLinks(toValue(variant), environment.value),
	})
}
