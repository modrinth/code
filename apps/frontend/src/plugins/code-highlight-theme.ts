import {
	CODE_HIGHLIGHT_THEME_STORAGE,
	DEFAULT_CODE_THEME,
	loadCodeHighlightThemeCss,
	SERVER_CODE_THEME_STYLE_ID,
	useCodeHighlightTheme,
} from '@modrinth/ui'
import { useLocalStorage } from '@vueuse/core'

import { getSettingsCookieOptions } from '~/composables/cookie-settings'

/**
 * The user's code highlighting theme. The browser keeps it in local storage, which keeps every tab
 * in step, and copies it into a cookie so server rendered pages already carry it (marking the page
 * and including the theme's stylesheet). The cookie is only ever written by the browser, never read
 * back, so its change events can't feed back into the theme.
 */
export default defineNuxtPlugin({
	name: 'code-highlight-theme',
	async setup(nuxtApp) {
		const cookie = useCookie<string>('code-highlight-theme', {
			...getSettingsCookieOptions<string>(),
			default: () => DEFAULT_CODE_THEME,
		})

		if (import.meta.server) {
			nuxtApp.vueApp.provide(CODE_HIGHLIGHT_THEME_STORAGE, cookie)
			const css = await loadCodeHighlightThemeCss(cookie.value)
			if (css) {
				useHead({
					htmlAttrs: { 'data-code-theme': cookie.value },
					style: [{ id: SERVER_CODE_THEME_STYLE_ID, innerHTML: css }],
				})
			}
			return
		}

		const theme = useLocalStorage('code-highlight-theme', cookie.value)
		watch(
			theme,
			(id) => {
				if (cookie.value !== id) cookie.value = id
			},
			{ immediate: true },
		)
		nuxtApp.vueApp.provide(CODE_HIGHLIGHT_THEME_STORAGE, theme)
		nuxtApp.vueApp.runWithContext(() => useCodeHighlightTheme().start())
	},
})
