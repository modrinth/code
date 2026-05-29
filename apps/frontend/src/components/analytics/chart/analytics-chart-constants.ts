import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

export const TITLE_BY_ANALYTICS_STAT: Record<AnalyticsDashboardStat, string> = {
	views: 'Views Over Time',
	downloads: 'Downloads Over Time',
	revenue: 'Revenue Over Time',
	playtime: 'Playtime Over Time',
}

export const ANALYTICS_DASHBOARD_STATS: readonly AnalyticsDashboardStat[] = [
	'views',
	'downloads',
	'revenue',
	'playtime',
]

export const TOP_GRAPH_DATASET_LIMIT = 8
export const GRAPH_RENDER_DATASET_LIMIT = 250
export const PREVIOUS_PERIOD_DATASET_ID_PREFIX = 'previous-period:'
export const PREVIOUS_PERIOD_BORDER_DASH = [6, 4]
export const PROJECT_VERSION_UPLOAD_DEDUPE_WINDOW_MS = 24 * 60 * 60 * 1000
export const ALL_PROJECTS_DATASET_ID = 'all'

export const PROJECT_EVENT_DATE_FORMATTER = new Intl.DateTimeFormat(undefined, {
	month: 'short',
	day: 'numeric',
	year: 'numeric',
})

export const MONETIZATION_LEGEND_ENTRY_ORDER = new Map([
	['breakdown:monetized', 0],
	['breakdown:unmonetized', 1],
])

export const VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUSES = [
	'approved',
	'unlisted',
	'private',
] as const satisfies readonly Labrinth.Projects.v2.ProjectStatus[]

export type VisibleProjectStatusChangeEventStatus =
	(typeof VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUSES)[number]

export const VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUS_SET =
	new Set<Labrinth.Projects.v2.ProjectStatus>(VISIBLE_PROJECT_STATUS_CHANGE_EVENT_STATUSES)

export const PROJECT_STATUS_EVENT_COPY: Record<VisibleProjectStatusChangeEventStatus, string> = {
	approved: 'Project approved',
	unlisted: 'Project unlisted',
	private: 'Project set to private',
}

export const LIGHT_LEGEND_PALETTE = [
	'hsl(152, 100%, 34%)',
	'hsl(26, 100%, 42%)',
	'hsl(202, 100%, 35%)',
	'hsl(327, 45%, 64%)',
	'hsl(41, 100%, 45%)',
	'hsl(250, 60%, 33%)',
	'hsl(170, 43%, 47%)',
	'hsl(330, 60%, 33%)',
	'hsl(46, 100%, 36%)',
	'hsl(167, 100%, 30%)',
	'hsl(343, 38%, 45%)',
	'hsl(222, 100%, 28%)',
	'hsl(270, 62%, 60%)',
	'hsl(32, 100%, 37%)',
	'hsl(349, 57%, 51%)',
	'hsl(191, 43%, 37%)',
]

export const DARK_LEGEND_PALETTE = [
	'hsl(145, 78%, 48%)',
	'hsl(41, 100%, 50%)',
	'hsl(202, 77%, 63%)',
	'hsl(323, 66%, 72%)',
	'hsl(56, 85%, 60%)',
	'hsl(255, 92%, 80%)',
	'hsl(12, 100%, 67%)',
	'hsl(176, 58%, 56%)',
	'hsl(60, 100%, 41%)',
	'hsl(165, 80%, 38%)',
	'hsl(341, 36%, 56%)',
	'hsl(226, 60%, 49%)',
	'hsl(252, 53%, 62%)',
	'hsl(75, 59%, 50%)',
	'hsl(195, 56%, 42%)',
	'hsl(30, 59%, 56%)',
]
