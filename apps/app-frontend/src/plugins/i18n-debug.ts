import {
	annotateFullDocument,
	clearAllAnnotations,
	hideKeyTooltip,
	I18N_DEBUG_KEY,
	type I18nDebugContext,
	initI18nDebugRuntime,
} from '@modrinth/ui'
import type { App } from 'vue'
import { reactive, ref, watch } from 'vue'

import { useTheming } from '@/store/theme'

export default {
	install(app: App) {
		const theming = useTheming()

		const enabled = ref(theming.featureFlags.i18n_debug ?? false)
		const keyReveal = ref(false)
		const registry = reactive(new Map()) as Map<
			string,
			{ key: string; value: string; defaultMessage?: string; timestamp: number }
		>
		const panelOpen = ref(false)

		const context: I18nDebugContext = { enabled, keyReveal, registry, panelOpen }
		app.provide(I18N_DEBUG_KEY, context)

		initI18nDebugRuntime(context)

		watch(
			() => theming.featureFlags.i18n_debug,
			(active) => {
				enabled.value = active
				if (!active) {
					keyReveal.value = false
					panelOpen.value = false
					document.body.classList.remove('i18n-debug')
					clearAllAnnotations()
					hideKeyTooltip()
					registry.clear()
				} else {
					document.body.classList.add('i18n-debug')
					annotateFullDocument(registry)
				}
			},
		)
	},
}
