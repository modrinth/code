import type { CrowdinMessages } from '#ui/composables'

export const uiLocaleModulesEager = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: true },
)
