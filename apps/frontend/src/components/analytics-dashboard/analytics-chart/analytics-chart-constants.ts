import type { Labrinth } from '@modrinth/api-client'

import type { AnalyticsDashboardStat } from '~/providers/analytics/analytics'

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

export const LIGHT_LEGEND_PALETTE = [
	'hsl(152, 100%, 34%)',
	'hsl(41, 79%, 46%)',
	'hsl(203, 76%, 64%)',
	'hsl(0, 93%, 62%)',
	'hsl(143, 66%, 29%)',
	'hsl(58, 89%, 25%)',
	'hsl(311, 64%, 49%)',
	'hsl(198, 91%, 32%)',
	'hsl(12, 88%, 27%)',
	'hsl(236, 61%, 60%)',
	'hsl(102, 59%, 74%)',
	'hsl(293, 76%, 79%)',
	'hsl(67, 99%, 41%)',
	'hsl(179, 100%, 50%)',
	'hsl(102, 100%, 61%)',
	'hsl(0, 100%, 32%)',
]

export const DARK_LEGEND_PALETTE = [
	'hsl(145, 78%, 48%)',
	'hsl(41, 79%, 46%)',
	'hsl(203, 76%, 64%)',
	'hsl(0, 93%, 62%)',
	'hsl(143, 66%, 29%)',
	'hsl(58, 94%, 45%)',
	'hsl(311, 64%, 49%)',
	'hsl(198, 91%, 32%)',
	'hsl(12, 88%, 27%)',
	'hsl(236, 61%, 60%)',
	'hsl(102, 59%, 74%)',
	'hsl(293, 76%, 79%)',
	'hsl(61, 92%, 33%)',
	'hsl(179, 100%, 50%)',
	'hsl(102, 100%, 61%)',
	'hsl(0, 100%, 32%)',
]
