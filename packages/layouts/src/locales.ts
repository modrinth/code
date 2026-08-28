import type { CrowdinMessages } from '@modrinth/ui'

export const layoutsLocaleModules = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)
