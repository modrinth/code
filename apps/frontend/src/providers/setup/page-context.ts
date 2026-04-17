import { provideModalBehavior, providePageContext } from '@modrinth/ui'
import { computed, ref } from 'vue'

import { useFeatureFlags } from '~/composables/featureFlags.ts'

export function setupPageContextProvider() {
	const cosmetics = useCosmetics()
	const featureFlags = useFeatureFlags()

	providePageContext({
		hierarchicalSidebarAvailable: ref(false),
		showAds: ref(false),
		featureFlags: {
			serverRamAsBytesAlwaysOn: computed(() => featureFlags.value.serverRamAsBytesAlwaysOn),
		},
		openExternalUrl: (url) => window.open(url, '_blank'),
	})
	provideModalBehavior({
		noblur: computed(() => !(cosmetics.value?.advancedRendering ?? true)),
	})
}
