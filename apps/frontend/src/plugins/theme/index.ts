import type { Labrinth } from '@modrinth/api-client'
import { prepareThemeColorTransition } from '@modrinth/ui'
import { ref, watch, watchEffect } from 'vue'

import { useNativeTheme } from './native-theme.ts'
import { usePreferredThemes } from './preferred-theme.ts'
import { useThemeSettings } from './theme-settings.ts'
import { isDarkTheme, type Theme } from './themes.ts'

export * from './themes.ts'

export default defineNuxtPlugin({
	name: 'theme',
	dependsOn: ['cosmetics'],
	setup(nuxtApp) {
		const $nativeTheme = useNativeTheme()

		const $preferredThemes = usePreferredThemes()

		function getPreferredNativeTheme() {
			const nativeTheme = $nativeTheme.value
			switch (nativeTheme) {
				case 'light':
					return $preferredThemes.light
				case 'dark':
				case 'unknown':
					if (import.meta.dev && import.meta.server && nativeTheme === 'unknown') {
						console.warn(
							'[theme] no client hint is available for request, using dark theme as default',
						)
					}

					return $preferredThemes.dark
			}
		}

		const $settings = useThemeSettings(() => getPreferredNativeTheme())
		const $preview = ref<Theme | 'system' | null>(null)
		const $active = computed(() => {
			if ($preview.value === null) return $settings.active
			return $preview.value === 'system' ? getPreferredNativeTheme() : $preview.value
		})

		useHead({ htmlAttrs: { class: () => [`${$active.value}-mode`] } })

		function syncTheme() {
			$settings.active =
				$settings.preferred === 'system' ? getPreferredNativeTheme() : $settings.preferred
		}

		function applyAccountAppearance(appearance: Labrinth.Users.v3.AppearancePreferences) {
			if (isDarkTheme(appearance.theme)) {
				$preferredThemes.dark = appearance.theme
			} else {
				$preferredThemes.light = appearance.theme
			}

			$settings.preferred = appearance.auto ? 'system' : appearance.theme

			const systemThemeIsUnknown =
				import.meta.server && $settings.preferred === 'system' && $nativeTheme.value === 'unknown'

			if (!systemThemeIsUnknown) syncTheme()
		}

		if (
			import.meta.server &&
			$settings.preferred === 'system' &&
			$nativeTheme.value !== 'unknown'
		) {
			// take advantage of the client hint
			syncTheme()
		}

		if (import.meta.client) {
			const $clientReady = ref(false)
			let themeColorTransitionsEnabled = false

			nuxtApp.hook('app:suspense:resolve', () => {
				$clientReady.value = true
			})

			watchEffect(() => {
				if (!$clientReady.value) return
				syncTheme()
				themeColorTransitionsEnabled = true
			})

			watch(
				$active,
				(theme, previousTheme) => {
					if (themeColorTransitionsEnabled && previousTheme && theme !== previousTheme) {
						prepareThemeColorTransition()
					}
				},
				{ flush: 'sync' },
			)
		}

		function cycle() {
			const nextTheme = isDarkTheme($settings.active)
				? $preferredThemes.light
				: $preferredThemes.dark

			$settings.preferred = nextTheme

			return nextTheme
		}

		return {
			provide: {
				theme: reactive({
					...toRefs($settings),
					active: $active,
					preview: $preview,
					/**
					 * Preferred themes for each mode.
					 */
					preferences: $preferredThemes,
					/**
					 * Current native (system) theme provided through client hint header or
					 * `prefers-color-scheme` media query.
					 */
					native: $nativeTheme,
					cycle,
					applyAccountAppearance,
				}),
			},
		}
	},
})
