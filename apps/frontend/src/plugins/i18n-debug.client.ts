import {
	annotateFullDocument,
	clearAllAnnotations,
	hideKeyTooltip,
	I18N_DEBUG_KEY,
	type I18nDebugContext,
	initI18nDebugRuntime,
} from '@modrinth/ui'

export default defineNuxtPlugin({
	name: 'i18n-debug',
	enforce: 'post',
	setup(nuxtApp) {
		const flags = useFeatureFlags()
		if (!flags.value.i18nDebug) return

		const enabled = ref(true)
		const keyReveal = ref(false)
		const registry = reactive(new Map()) as Map<
			string,
			{ key: string; value: string; defaultMessage?: string; timestamp: number }
		>
		const panelOpen = ref(false)

		const context: I18nDebugContext = { enabled, keyReveal, registry, panelOpen }
		nuxtApp.vueApp.provide(I18N_DEBUG_KEY, context)

		nuxtApp.hook('app:mounted', () => {
			initI18nDebugRuntime(context)
		})

		watch(
			() => flags.value.i18nDebug,
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
})
