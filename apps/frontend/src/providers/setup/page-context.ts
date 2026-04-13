import { provideModalBehavior, providePageContext } from '@modrinth/ui'
import { computed, ref } from 'vue'

export function setupPageContextProvider() {
	const cosmetics = useCosmetics()

	providePageContext({
		hierarchicalSidebarAvailable: ref(false),
		showAds: ref(false),
		openExternalUrl: (url) => window.open(url, '_blank'),
	})
	provideModalBehavior({
		noblur: computed(() => !(cosmetics.value?.advancedRendering ?? true)),
	})
}
