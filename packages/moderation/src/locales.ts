import type { CrowdinMessages } from '@icarus/ui'

export const moderationLocaleModules = import.meta.glob<{ default: CrowdinMessages }>(
	'./locales/*/index.json',
	{ eager: false },
)
