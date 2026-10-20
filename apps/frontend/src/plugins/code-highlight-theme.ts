import {
	CODE_HIGHLIGHT_THEME_STORAGE,
	DEFAULT_CODE_THEME,
	loadCodeHighlightThemeCss,
	SERVER_CODE_THEME_STYLE_ID,
	useCodeHighlightTheme,
} from '@modrinth/ui'

import { getSettingsCookieOptions } from '~/composables/cookie-settings'

/**
 * The user's code highlighting theme, kept in a cookie so server rendered pages already carry it
 * (marking the page and including the theme's stylesheet), then applied by the browser.
 */
export default defineNuxtPlugin({
	name: 'code-highlight-theme',
	async setup(nuxtApp) {
		const cookie = useCookie<string>('code-highlight-theme', {
			...getSettingsCookieOptions<string>(),
			default: () => DEFAULT_CODE_THEME,
		})
		nuxtApp.vueApp.provide(CODE_HIGHLIGHT_THEME_STORAGE, cookie)

		if (import.meta.server) {
			const css = await loadCodeHighlightThemeCss(cookie.value)
			if (css) {
				useHead({
					htmlAttrs: { 'data-code-theme': cookie.value },
					style: [{ id: SERVER_CODE_THEME_STYLE_ID, innerHTML: css }],
				})
			}
			return
		}

		nuxtApp.vueApp.runWithContext(() => useCodeHighlightTheme().start())
	},
})
