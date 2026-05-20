import type { CrowdinMessages } from './composables/i18n'

export const uiLocaleModulesEager = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: true },
)
