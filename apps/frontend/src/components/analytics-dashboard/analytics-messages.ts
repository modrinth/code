import { defineMessages, getLoaderMessage, type VIntlFormatters } from '@modrinth/ui'

import type {
	AnalyticsBreakdownPreset,
	AnalyticsDashboardStat,
	AnalyticsGroupByPreset,
} from '~/providers/analytics/analytics'

export type FormatMessage = VIntlFormatters['formatMessage']
export type AnalyticsBreakdownItemType =
	| 'country'
	| 'downloadReason'
	| 'downloadSource'
	| 'gameVersion'
	| 'loader'
	| 'member'
	| 'monetization'
	| 'project'
	| 'projectVersion'
	| 'other'

export const analyticsMessages = defineMessages({
	title: {
		id: 'analytics.title',
		defaultMessage: 'Analytics',
	},
	resetButton: {
		id: 'analytics.action.reset',
		defaultMessage: 'Reset',
	},
	refreshButton: {
		id: 'analytics.action.refresh',
		defaultMessage: 'Refresh',
	},
	saveButton: {
		id: 'analytics.action.save',
		defaultMessage: 'Save',
	},
	cancelButton: {
		id: 'analytics.action.cancel',
		defaultMessage: 'Cancel',
	},
	fetchingResults: {
		id: 'analytics.loading.fetching-results',
		defaultMessage: 'Fetching results...',
	},
	allProjects: {
		id: 'analytics.project.all',
		defaultMessage: 'All projects',
	},
	yourProjects: {
		id: 'analytics.project.your',
		defaultMessage: 'Your projects',
	},
	userProjects: {
		id: 'analytics.project.user',
		defaultMessage: "{username}'s projects",
	},
	selectProjects: {
		id: 'analytics.project.select',
		defaultMessage: 'Select projects',
	},
	projectCount: {
		id: 'analytics.project.count',
		defaultMessage: '{count, plural, one {# project} other {# projects}}',
	},
	projectIconAlt: {
		id: 'analytics.project.icon-alt',
		defaultMessage: '{name} Icon',
	},
	noDataAvailable: {
		id: 'analytics.empty.no-data',
		defaultMessage: 'No data available',
	},
	noDataAvailableForAnalytics: {
		id: 'analytics.empty.no-data-for-analytics',
		defaultMessage: 'No data available for analytics',
	},
	noProjectsAvailable: {
		id: 'analytics.empty.no-projects',
		defaultMessage: 'No projects available',
	},
	noProjectsAvailableForAnalytics: {
		id: 'analytics.empty.no-projects-for-analytics',
		defaultMessage: 'No projects available for analytics',
	},
	selectAtLeastOneProject: {
		id: 'analytics.empty.select-project',
		defaultMessage: 'Select at least one project to view data',
	},
	unknown: {
		id: 'analytics.value.unknown',
		defaultMessage: 'Unknown',
	},
	noDependent: {
		id: 'analytics.value.no-dependent',
		defaultMessage: 'None',
	},
	noDependentTooltip: {
		id: 'analytics.value.no-dependent-tooltip',
		defaultMessage: 'Downloaded for reasons other than being a dependency',
	},
	unknownDependentTooltip: {
		id: 'analytics.value.unknown-dependent-tooltip',
		defaultMessage:
			"There's no metadata to determine which dependent project this download attributes to.",
	},
	other: {
		id: 'analytics.value.other',
		defaultMessage: 'Other',
	},
	none: {
		id: 'analytics.value.none',
		defaultMessage: 'None',
	},
	noBreakdown: {
		id: 'analytics.breakdown.none.selected',
		defaultMessage: 'No breakdown',
	},
	breakdownBy: {
		id: 'analytics.breakdown.selected',
		defaultMessage: 'Breakdown by {breakdown}',
	},
	projectLabel: {
		id: 'analytics.query.label.project',
		defaultMessage: 'Project:',
	},
	timeframeLabel: {
		id: 'analytics.query.label.timeframe',
		defaultMessage: 'Timeframe:',
	},
	groupedByLabel: {
		id: 'analytics.query.label.grouped-by',
		defaultMessage: 'Grouped by',
	},
	breakdownLabel: {
		id: 'analytics.query.label.breakdown',
		defaultMessage: 'Breakdown:',
	},
	breakdownGroupsButton: {
		id: 'analytics.breakdown-groups.button',
		defaultMessage: 'Group series',
	},
	breakdownGroupsNoGrouping: {
		id: 'analytics.breakdown-groups.none',
		defaultMessage: 'No grouping',
	},
	breakdownGroupsCreate: {
		id: 'analytics.breakdown-groups.create',
		defaultMessage: 'Create group',
	},
	breakdownGroupsEdit: {
		id: 'analytics.breakdown-groups.edit',
		defaultMessage: 'Edit group',
	},
	breakdownGroupsDelete: {
		id: 'analytics.breakdown-groups.delete',
		defaultMessage: 'Delete group',
	},
	breakdownGroupsDeleteTitle: {
		id: 'analytics.breakdown-groups.delete-title',
		defaultMessage: 'Delete {name}?',
	},
	breakdownGroupsDeleteDescription: {
		id: 'analytics.breakdown-groups.delete-description',
		defaultMessage: 'This saved breakdown group will be permanently deleted from this browser.',
	},
	breakdownGroupsModalCreateTitle: {
		id: 'analytics.breakdown-groups.modal.create-title',
		defaultMessage: 'Create breakdown group',
	},
	breakdownGroupsModalEditTitle: {
		id: 'analytics.breakdown-groups.modal.edit-title',
		defaultMessage: 'Edit breakdown group',
	},
	breakdownGroupsGroupName: {
		id: 'analytics.breakdown-groups.group-name',
		defaultMessage: 'Group name',
	},
	breakdownGroupsGroupNamePlaceholder: {
		id: 'analytics.breakdown-groups.group-name-placeholder',
		defaultMessage: 'For example, Platform families',
	},
	breakdownGroupsSeriesName: {
		id: 'analytics.breakdown-groups.series-name',
		defaultMessage: 'Series name',
	},
	breakdownGroupsSeriesValues: {
		id: 'analytics.breakdown-groups.series-values',
		defaultMessage: 'Breakdown values',
	},
	breakdownGroupsSelectValues: {
		id: 'analytics.breakdown-groups.select-values',
		defaultMessage: 'Select values',
	},
	breakdownGroupsAddSeries: {
		id: 'analytics.breakdown-groups.add-series',
		defaultMessage: 'Add series',
	},
	breakdownGroupsRemoveSeries: {
		id: 'analytics.breakdown-groups.remove-series',
		defaultMessage: 'Remove series',
	},
	breakdownGroupsOtherDescription: {
		id: 'analytics.breakdown-groups.other-description',
		defaultMessage: '{count, plural, one {# unassigned value} other {# unassigned values}}',
	},
	breakdownGroupsUnavailableValue: {
		id: 'analytics.breakdown-groups.unavailable-value',
		defaultMessage: '{value} (not in current results)',
	},
	breakdownGroupsNameRequired: {
		id: 'analytics.breakdown-groups.validation.name-required',
		defaultMessage: 'Enter a group name.',
	},
	breakdownGroupsNameDuplicate: {
		id: 'analytics.breakdown-groups.validation.name-duplicate',
		defaultMessage: 'A group with this name already exists for this breakdown.',
	},
	breakdownGroupsSeriesRequired: {
		id: 'analytics.breakdown-groups.validation.series-required',
		defaultMessage: 'Add at least one custom series.',
	},
	breakdownGroupsSeriesNameRequired: {
		id: 'analytics.breakdown-groups.validation.series-name-required',
		defaultMessage: 'Every custom series needs a name.',
	},
	breakdownGroupsSeriesNameDuplicate: {
		id: 'analytics.breakdown-groups.validation.series-name-duplicate',
		defaultMessage: 'Custom series names must be unique.',
	},
	breakdownGroupsSeriesValuesRequired: {
		id: 'analytics.breakdown-groups.validation.series-values-required',
		defaultMessage: 'Assign at least one breakdown value to every custom series.',
	},
	addFilterButton: {
		id: 'analytics.query.filter.add',
		defaultMessage: 'Add filter',
	},
	addButton: {
		id: 'analytics.action.add',
		defaultMessage: 'Add',
	},
	downloadsSuffix: {
		id: 'analytics.downloads.suffix',
		defaultMessage: 'downloads',
	},
	projectsAbove: {
		id: 'analytics.threshold.projects-above',
		defaultMessage: 'Projects above',
	},
	countriesAbove: {
		id: 'analytics.threshold.countries-above',
		defaultMessage: 'Countries above',
	},
	projectVersionsAbove: {
		id: 'analytics.threshold.project-versions-above',
		defaultMessage: 'Project versions above',
	},
	gameVersionsAbove: {
		id: 'analytics.threshold.game-versions-above',
		defaultMessage: 'Game versions above',
	},
	projectDownloadsThresholdAria: {
		id: 'analytics.threshold.project-downloads-aria',
		defaultMessage: 'Project downloads threshold',
	},
	countryDownloadsThresholdAria: {
		id: 'analytics.threshold.country-downloads-aria',
		defaultMessage: 'Country downloads threshold',
	},
	projectVersionDownloadsThresholdAria: {
		id: 'analytics.threshold.project-version-downloads-aria',
		defaultMessage: 'Project version downloads threshold',
	},
	gameVersionDownloadsThresholdAria: {
		id: 'analytics.threshold.game-version-downloads-aria',
		defaultMessage: 'Game version downloads threshold',
	},
	loadingOptions: {
		id: 'analytics.options.loading',
		defaultMessage: 'Loading...',
	},
	searchCountriesPlaceholder: {
		id: 'analytics.filter.search.countries',
		defaultMessage: 'Search countries...',
	},
	searchDownloadSourcesPlaceholder: {
		id: 'analytics.filter.search.download-sources',
		defaultMessage: 'Search download sources...',
	},
	searchProjectVersionsPlaceholder: {
		id: 'analytics.filter.search.project-versions',
		defaultMessage: 'Search project versions...',
	},
	searchDependentProjectsPlaceholder: {
		id: 'analytics.filter.search.dependent-projects',
		defaultMessage: 'Search projects...',
	},
	searchMembersPlaceholder: {
		id: 'analytics.filter.search.members',
		defaultMessage: 'Search members...',
	},
	searchVersionsPlaceholder: {
		id: 'analytics.filter.search.versions',
		defaultMessage: 'Search versions...',
	},
	gameVersionTypeAria: {
		id: 'analytics.filter.game-version-type',
		defaultMessage: 'Game version type',
	},
	releaseTab: {
		id: 'analytics.filter.game-version-type.release',
		defaultMessage: 'Release',
	},
	allTab: {
		id: 'analytics.filter.game-version-type.all',
		defaultMessage: 'All',
	},
})

export const analyticsStatMessages = defineMessages({
	views: {
		id: 'analytics.stat.views',
		defaultMessage: 'Views',
	},
	downloads: {
		id: 'analytics.stat.downloads',
		defaultMessage: 'Downloads',
	},
	revenue: {
		id: 'analytics.stat.revenue',
		defaultMessage: 'Revenue',
	},
	playtime: {
		id: 'analytics.stat.playtime',
		defaultMessage: 'Playtime',
	},
})

export const analyticsGraphTitleMessages = defineMessages({
	views: {
		id: 'analytics.graph.title.views',
		defaultMessage: 'Views Over Time',
	},
	downloads: {
		id: 'analytics.graph.title.downloads',
		defaultMessage: 'Downloads Over Time',
	},
	revenue: {
		id: 'analytics.graph.title.revenue',
		defaultMessage: 'Revenue Over Time',
	},
	playtime: {
		id: 'analytics.graph.title.playtime',
		defaultMessage: 'Playtime Over Time',
	},
})

export const analyticsStatCardMessages = defineMessages({
	monetizationBannerTitle: {
		id: 'analytics.stat.monetization-banner.title',
		defaultMessage: 'How does monetization work?',
	},
	monetizationBannerBody: {
		id: 'analytics.stat.monetization-banner.body',
		defaultMessage:
			'Only views and downloads made through Modrinth are eligible for monetization and must pass fraud-prevention filtering. Modrinth App downloads also require the user to be logged in. Because all projects have a similar ratio of monetized downloads, your revenue would not meaningfully change if all downloads were counted.',
	},
	monetizationBannerLearnMore: {
		id: 'analytics.stat.monetization-banner.learn-more',
		defaultMessage: 'Learn more',
	},
	revenueValue: {
		id: 'analytics.stat.revenue-value',
		defaultMessage: '${value}',
	},
	playtimeHours: {
		id: 'analytics.stat.playtime-hours',
		defaultMessage: '{hours} hrs',
	},
	unavailableTooltip: {
		id: 'analytics.stat.unavailable-tooltip',
		defaultMessage: 'Stat unavailable for current query',
	},
	unavailableLabel: {
		id: 'analytics.stat.unavailable',
		defaultMessage: 'N/A',
	},
	previousPeriodComparison: {
		id: 'analytics.stat.previous-period-comparison',
		defaultMessage: 'vs prev. period',
	},
	previousPeriodComparisonShort: {
		id: 'analytics.stat.previous-period-comparison-short',
		defaultMessage: 'vs prev.',
	},
})

export const analyticsGroupByMessages = defineMessages({
	oneHour: {
		id: 'analytics.group-by.1h',
		defaultMessage: '1h',
	},
	sixHours: {
		id: 'analytics.group-by.6h',
		defaultMessage: '6h',
	},
	day: {
		id: 'analytics.group-by.day',
		defaultMessage: 'Day',
	},
	week: {
		id: 'analytics.group-by.week',
		defaultMessage: 'Week',
	},
	month: {
		id: 'analytics.group-by.month',
		defaultMessage: 'Month',
	},
	year: {
		id: 'analytics.group-by.year',
		defaultMessage: 'Year',
	},
	date: {
		id: 'analytics.group-by.date',
		defaultMessage: 'Date',
	},
	groupByHour: {
		id: 'analytics.group-by.selected.hour',
		defaultMessage: 'Group by hour',
	},
	groupBySixHours: {
		id: 'analytics.group-by.selected.six-hours',
		defaultMessage: 'Group by 6 hours',
	},
	groupByDay: {
		id: 'analytics.group-by.selected.day',
		defaultMessage: 'Group by day',
	},
	groupByWeek: {
		id: 'analytics.group-by.selected.week',
		defaultMessage: 'Group by week',
	},
	groupByMonth: {
		id: 'analytics.group-by.selected.month',
		defaultMessage: 'Group by month',
	},
	groupByYear: {
		id: 'analytics.group-by.selected.year',
		defaultMessage: 'Group by year',
	},
})

export const analyticsBreakdownMessages = defineMessages({
	breakdown: {
		id: 'analytics.breakdown.generic',
		defaultMessage: 'Breakdown',
	},
	project: {
		id: 'analytics.breakdown.project',
		defaultMessage: 'Project',
	},
	country: {
		id: 'analytics.breakdown.country',
		defaultMessage: 'Country',
	},
	monetization: {
		id: 'analytics.breakdown.monetization',
		defaultMessage: 'Monetization',
	},
	userAgent: {
		id: 'analytics.breakdown.download-source',
		defaultMessage: 'Download source',
	},
	downloadReason: {
		id: 'analytics.breakdown.download-reason',
		defaultMessage: 'Download reason',
	},
	members: {
		id: 'analytics.breakdown.members',
		defaultMessage: 'Member',
	},
	dependentProjectDownload: {
		id: 'analytics.breakdown.dependent-project-download',
		defaultMessage: 'Dependent project',
	},
	dependentProjectType: {
		id: 'analytics.breakdown.dependent-project-type',
		defaultMessage: 'Dependent project type',
	},
	dependentOn: {
		id: 'analytics.breakdown.dependent-on',
		defaultMessage: 'Dependent on',
	},
	versionId: {
		id: 'analytics.breakdown.project-version',
		defaultMessage: 'Project version',
	},
	loader: {
		id: 'analytics.breakdown.loader',
		defaultMessage: 'Loader',
	},
	gameVersion: {
		id: 'analytics.breakdown.game-version',
		defaultMessage: 'Game version',
	},
	projectStatus: {
		id: 'analytics.breakdown.project-status',
		defaultMessage: 'Project status',
	},
})

export const analyticsMonetizationMessages = defineMessages({
	monetized: {
		id: 'analytics.value.monetized',
		defaultMessage: 'Monetized',
	},
	unmonetized: {
		id: 'analytics.value.unmonetized',
		defaultMessage: 'Unmonetized',
	},
})

export const analyticsDownloadReasonMessages = defineMessages({
	standalone: {
		id: 'analytics.download-reason.standalone',
		defaultMessage: 'Standalone',
	},
	dependency: {
		id: 'analytics.download-reason.dependency',
		defaultMessage: 'Dependency',
	},
	modpack: {
		id: 'analytics.download-reason.modpack',
		defaultMessage: 'Modpack',
	},
	update: {
		id: 'analytics.download-reason.update',
		defaultMessage: 'Update',
	},
})

export const analyticsDownloadSourceMessages = defineMessages({
	website: {
		id: 'analytics.download-source.website',
		defaultMessage: 'Modrinth Website',
	},
	app: {
		id: 'analytics.download-source.app',
		defaultMessage: 'Modrinth App',
	},
})

export const analyticsProjectStatusMessages = defineMessages({
	approved: {
		id: 'analytics.project-status.approved',
		defaultMessage: 'Approved',
	},
	archived: {
		id: 'analytics.project-status.archived',
		defaultMessage: 'Archived',
	},
	rejected: {
		id: 'analytics.project-status.rejected',
		defaultMessage: 'Rejected',
	},
	draft: {
		id: 'analytics.project-status.draft',
		defaultMessage: 'Draft',
	},
	unlisted: {
		id: 'analytics.project-status.unlisted',
		defaultMessage: 'Unlisted',
	},
	withheld: {
		id: 'analytics.project-status.withheld',
		defaultMessage: 'Withheld',
	},
	private: {
		id: 'analytics.project-status.private',
		defaultMessage: 'Private',
	},
	other: {
		id: 'analytics.project-status.other',
		defaultMessage: 'Other',
	},
})

export const analyticsTableMessages = defineMessages({
	searchPlaceholder: {
		id: 'analytics.table.search.placeholder',
		defaultMessage: 'Search...',
	},
	exportCsvButton: {
		id: 'analytics.table.export-csv',
		defaultMessage: 'Export CSV',
	},
	cumulativeCsv: {
		id: 'analytics.table.export.cumulative',
		defaultMessage: 'Cumulative',
	},
	groupedCsv: {
		id: 'analytics.table.export.grouped',
		defaultMessage: 'Grouped by {groupBy}',
	},
	noMatchingRows: {
		id: 'analytics.table.empty.no-matching-rows',
		defaultMessage: 'No matching analytics rows',
	},
	paginationSummary: {
		id: 'analytics.table.pagination.summary',
		defaultMessage: 'Showing {start} to {end} of {total}',
	},
	playtimeSecondsHeader: {
		id: 'analytics.table.csv.header.playtime-seconds',
		defaultMessage: 'Playtime (seconds)',
	},
	csvSelectedRange: {
		id: 'analytics.table.csv.selected-range',
		defaultMessage: 'Selected Range',
	},
	csvDateRange: {
		id: 'analytics.table.csv.date-range',
		defaultMessage: '{start} to {end}',
	},
	csvFilename: {
		id: 'analytics.table.csv.filename',
		defaultMessage: 'Modrinth Analytics {breakdown} Breakdown - {dateRange}',
	},
	durationDays: {
		id: 'analytics.table.duration.days',
		defaultMessage: '{count, plural, one {# day} other {# days}}',
	},
	durationHours: {
		id: 'analytics.table.duration.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	durationMinutes: {
		id: 'analytics.table.duration.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
})

export const analyticsChartMessages = defineMessages({
	selectTableItemsEmpty: {
		id: 'analytics.chart.empty.select-table-items',
		defaultMessage: 'Select items from table below to visualize your data.',
	},
	showLimited: {
		id: 'analytics.chart.action.show-limited',
		defaultMessage: 'Show limited',
	},
	showAll: {
		id: 'analytics.chart.action.show-all',
		defaultMessage: 'Show all',
	},
	showTopEight: {
		id: 'analytics.chart.action.show-top-eight',
		defaultMessage: 'Show top 8',
	},
	dependentOnProjectTooltip: {
		id: 'analytics.chart.tooltip.dependent-on-project',
		defaultMessage: 'Dependent on {project}',
	},
	dependentProjectVersionTooltip: {
		id: 'analytics.chart.tooltip.dependent-project-version',
		defaultMessage: '{dependentProject} dependent on {dependencyProject}, {version}',
	},
	tableSelectionLimited: {
		id: 'analytics.chart.table-selection.limited',
		defaultMessage:
			'Showing {limit} {itemType, select, project {{limit, plural, one {project} other {projects}}} country {{limit, plural, one {country} other {countries}}} monetization {{limit, plural, one {monetization value} other {monetization values}}} downloadSource {{limit, plural, one {download source} other {download sources}}} downloadReason {{limit, plural, one {download reason} other {download reasons}}} member {{limit, plural, one {member} other {members}}} projectVersion {{limit, plural, one {project version} other {project versions}}} loader {{limit, plural, one {loader} other {loaders}}} gameVersion {{limit, plural, one {game version} other {game versions}}} other {{limit, plural, one {item} other {items}}}} from table',
	},
	tableSelectionAll: {
		id: 'analytics.chart.table-selection.all',
		defaultMessage:
			'Showing all {itemType, select, project {{count, plural, one {project} other {projects}}} country {{count, plural, one {country} other {countries}}} monetization {{count, plural, one {monetization value} other {monetization values}}} downloadSource {{count, plural, one {download source} other {download sources}}} downloadReason {{count, plural, one {download reason} other {download reasons}}} member {{count, plural, one {member} other {members}}} projectVersion {{count, plural, one {project version} other {project versions}}} loader {{count, plural, one {loader} other {loaders}}} gameVersion {{count, plural, one {game version} other {game versions}}} other {{count, plural, one {item} other {items}}}} from table',
	},
	tableSelectionTop: {
		id: 'analytics.chart.table-selection.top',
		defaultMessage:
			'Showing top {count} {itemType, select, project {{count, plural, one {project} other {projects}}} country {{count, plural, one {country} other {countries}}} monetization {{count, plural, one {monetization value} other {monetization values}}} downloadSource {{count, plural, one {download source} other {download sources}}} downloadReason {{count, plural, one {download reason} other {download reasons}}} member {{count, plural, one {member} other {members}}} projectVersion {{count, plural, one {project version} other {project versions}}} loader {{count, plural, one {loader} other {loaders}}} gameVersion {{count, plural, one {game version} other {game versions}}} other {{count, plural, one {item} other {items}}}} from table',
	},
	tableSelectionCount: {
		id: 'analytics.chart.table-selection.count',
		defaultMessage:
			'Showing {count} {itemType, select, project {{count, plural, one {project} other {projects}}} country {{count, plural, one {country} other {countries}}} monetization {{count, plural, one {monetization value} other {monetization values}}} downloadSource {{count, plural, one {download source} other {download sources}}} downloadReason {{count, plural, one {download reason} other {download reasons}}} member {{count, plural, one {member} other {members}}} projectVersion {{count, plural, one {project version} other {project versions}}} loader {{count, plural, one {loader} other {loaders}}} gameVersion {{count, plural, one {game version} other {game versions}}} other {{count, plural, one {item} other {items}}}} from table',
	},
	lineView: {
		id: 'analytics.chart.view.line',
		defaultMessage: 'Line',
	},
	areaView: {
		id: 'analytics.chart.view.area',
		defaultMessage: 'Area',
	},
	barView: {
		id: 'analytics.chart.view.bar',
		defaultMessage: 'Bar',
	},
	controlsButton: {
		id: 'analytics.chart.controls.button',
		defaultMessage: 'Controls',
	},
	controlsAria: {
		id: 'analytics.chart.controls.aria',
		defaultMessage: 'Analytics graph controls, {activeCount}',
	},
	controlsDialogAria: {
		id: 'analytics.chart.controls.dialog-aria',
		defaultMessage: 'Analytics graph controls',
	},
	activeControlCount: {
		id: 'analytics.chart.controls.active-count',
		defaultMessage: '{count} active',
	},
	displayControls: {
		id: 'analytics.chart.controls.display',
		defaultMessage: 'Display',
	},
	previousPeriod: {
		id: 'analytics.chart.controls.previous-period',
		defaultMessage: 'Previous period',
	},
	ratio: {
		id: 'analytics.chart.controls.ratio',
		defaultMessage: 'Ratio',
	},
	annotations: {
		id: 'analytics.chart.controls.annotations',
		defaultMessage: 'Annotations',
	},
	projectEvents: {
		id: 'analytics.chart.controls.project-events',
		defaultMessage: 'Project events',
	},
	modrinthEvents: {
		id: 'analytics.chart.controls.modrinth-events',
		defaultMessage: 'Modrinth events',
	},
	noProjectEvents: {
		id: 'analytics.chart.controls.no-project-events',
		defaultMessage: 'No project events in graph.',
	},
	noModrinthEvents: {
		id: 'analytics.chart.controls.no-modrinth-events',
		defaultMessage: 'No Modrinth events in graph.',
	},
	viewMonetizedAnalyticsDetails: {
		id: 'analytics.chart.legend.monetization-details.aria',
		defaultMessage: 'View monetized analytics details',
	},
	monetizedAnalyticsDetails: {
		id: 'analytics.chart.legend.monetization-details.title',
		defaultMessage: 'Monetized analytics details',
	},
	monetizedAnalyticsDetailsDescription: {
		id: 'analytics.chart.legend.monetization-details.description',
		defaultMessage:
			'Only views and downloads made through Modrinth count toward monetization, and downloads require users to be logged in.',
	},
	previousPeriodSuffix: {
		id: 'analytics.chart.legend.previous-period-suffix',
		defaultMessage: '{name} (Prev.)',
	},
	previousPeriodShort: {
		id: 'analytics.chart.tooltip.previous-period-short',
		defaultMessage: '(prev.)',
	},
	tooltipPinned: {
		id: 'analytics.chart.tooltip.pinned',
		defaultMessage: 'Chart tooltip pinned',
	},
	pinned: {
		id: 'analytics.chart.tooltip.pinned-aria',
		defaultMessage: 'Pinned',
	},
	total: {
		id: 'analytics.chart.tooltip.total',
		defaultMessage: 'Total',
	},
	showEntryInGraph: {
		id: 'analytics.chart.tooltip.show-entry',
		defaultMessage: 'Show {name} in graph',
	},
	hideEntryInGraph: {
		id: 'analytics.chart.tooltip.hide-entry',
		defaultMessage: 'Hide {name} in graph',
	},
	durationDays: {
		id: 'analytics.chart.tooltip.duration.days',
		defaultMessage: '{count, plural, one {# day} other {# days}}',
	},
	durationHours: {
		id: 'analytics.chart.tooltip.duration.hours',
		defaultMessage: '{count, plural, one {# hour} other {# hours}}',
	},
	durationMinutes: {
		id: 'analytics.chart.tooltip.duration.minutes',
		defaultMessage: '{count, plural, one {# minute} other {# minutes}}',
	},
	playtimeAxisHours: {
		id: 'analytics.chart.axis.playtime-hours',
		defaultMessage: '{hours} h',
	},
	renderLimitHeader: {
		id: 'analytics.chart.render-limit.header',
		defaultMessage: 'Show all {count} lines in graph?',
	},
	renderLimitDescription: {
		id: 'analytics.chart.render-limit.description',
		defaultMessage: 'Showing all selected lines from table may degrade page performance.',
	},
	cancelButton: {
		id: 'analytics.action.cancel',
		defaultMessage: 'Cancel',
	},
	analyticsEventsCount: {
		id: 'analytics.chart.events.count-aria',
		defaultMessage: '{count, plural, one {# analytics event} other {# analytics events}}',
	},
	seeAnnouncement: {
		id: 'analytics.chart.events.see-announcement',
		defaultMessage: 'See announcement',
	},
	projectEventTitle: {
		id: 'analytics.chart.events.project-title',
		defaultMessage: '<project>{projectName}</project>: {title}',
	},
})

export const analyticsProjectEventMessages = defineMessages({
	versionReleased: {
		id: 'analytics.project-event.version-released',
		defaultMessage: '{version} released',
	},
	versionUploaded: {
		id: 'analytics.project-event.version-uploaded',
		defaultMessage: 'Version uploaded',
	},
	projectApproved: {
		id: 'analytics.project-event.project-approved',
		defaultMessage: 'Project approved',
	},
	projectUnlisted: {
		id: 'analytics.project-event.project-unlisted',
		defaultMessage: 'Project unlisted',
	},
	projectPrivate: {
		id: 'analytics.project-event.project-private',
		defaultMessage: 'Project set to private',
	},
	projectStatusChanged: {
		id: 'analytics.project-event.project-status-changed',
		defaultMessage: 'Project status changed',
	},
})

export function formatAnalyticsStatLabel(
	stat: AnalyticsDashboardStat,
	formatMessage: FormatMessage,
): string {
	return formatMessage(analyticsStatMessages[stat])
}

export function formatAnalyticsGraphTitle(
	stat: AnalyticsDashboardStat,
	formatMessage: FormatMessage,
): string {
	return formatMessage(analyticsGraphTitleMessages[stat])
}

export function formatAnalyticsGroupByLabel(
	groupBy: AnalyticsGroupByPreset,
	formatMessage: FormatMessage,
): string {
	switch (groupBy) {
		case '1h':
			return formatMessage(analyticsGroupByMessages.oneHour)
		case '6h':
			return formatMessage(analyticsGroupByMessages.sixHours)
		case 'day':
			return formatMessage(analyticsGroupByMessages.day)
		case 'week':
			return formatMessage(analyticsGroupByMessages.week)
		case 'month':
			return formatMessage(analyticsGroupByMessages.month)
		case 'year':
			return formatMessage(analyticsGroupByMessages.year)
		default:
			return formatMessage(analyticsGroupByMessages.date)
	}
}

export function formatAnalyticsGroupBySelectedLabel(
	groupBy: AnalyticsGroupByPreset,
	formatMessage: FormatMessage,
): string {
	switch (groupBy) {
		case '1h':
			return formatMessage(analyticsGroupByMessages.groupByHour)
		case '6h':
			return formatMessage(analyticsGroupByMessages.groupBySixHours)
		case 'day':
			return formatMessage(analyticsGroupByMessages.groupByDay)
		case 'week':
			return formatMessage(analyticsGroupByMessages.groupByWeek)
		case 'month':
			return formatMessage(analyticsGroupByMessages.groupByMonth)
		case 'year':
			return formatMessage(analyticsGroupByMessages.groupByYear)
		default:
			return formatMessage(analyticsGroupByMessages.groupByDay)
	}
}

export function formatAnalyticsBreakdownLabel(
	breakdown: AnalyticsBreakdownPreset,
	formatMessage: FormatMessage,
): string {
	switch (breakdown) {
		case 'none':
		case 'project':
			return formatMessage(analyticsBreakdownMessages.project)
		case 'country':
			return formatMessage(analyticsBreakdownMessages.country)
		case 'monetization':
			return formatMessage(analyticsBreakdownMessages.monetization)
		case 'user_agent':
			return formatMessage(analyticsBreakdownMessages.userAgent)
		case 'download_reason':
			return formatMessage(analyticsBreakdownMessages.downloadReason)
		case 'user_id':
			return formatMessage(analyticsBreakdownMessages.members)
		case 'dependent_project_download':
			return formatMessage(analyticsBreakdownMessages.dependentProjectDownload)
		case 'version_id':
			return formatMessage(analyticsBreakdownMessages.versionId)
		case 'loader':
			return formatMessage(analyticsBreakdownMessages.loader)
		case 'game_version':
			return formatMessage(analyticsBreakdownMessages.gameVersion)
		default:
			return formatMessage(analyticsBreakdownMessages.breakdown)
	}
}

export function getAnalyticsBreakdownItemType(
	breakdowns: readonly AnalyticsBreakdownPreset[],
): AnalyticsBreakdownItemType {
	if (breakdowns.length !== 1) {
		return 'other'
	}

	switch (breakdowns[0]) {
		case 'project':
			return 'project'
		case 'country':
			return 'country'
		case 'monetization':
			return 'monetization'
		case 'user_agent':
			return 'downloadSource'
		case 'download_reason':
			return 'downloadReason'
		case 'user_id':
			return 'member'
		case 'dependent_project_download':
			return 'project'
		case 'version_id':
			return 'projectVersion'
		case 'loader':
			return 'loader'
		case 'game_version':
			return 'gameVersion'
		default:
			return 'other'
	}
}

export function formatAnalyticsMonetizationLabel(
	value: string,
	formatMessage: FormatMessage,
): string {
	switch (value.trim().toLowerCase()) {
		case 'monetized':
			return formatMessage(analyticsMonetizationMessages.monetized)
		case 'unmonetized':
			return formatMessage(analyticsMonetizationMessages.unmonetized)
		default:
			return value
	}
}

export function formatAnalyticsDownloadReasonLabel(
	reason: string,
	formatMessage: FormatMessage,
): string {
	switch (reason.trim().toLowerCase()) {
		case 'standalone':
			return formatMessage(analyticsDownloadReasonMessages.standalone)
		case 'dependency':
			return formatMessage(analyticsDownloadReasonMessages.dependency)
		case 'modpack':
			return formatMessage(analyticsDownloadReasonMessages.modpack)
		case 'update':
			return formatMessage(analyticsDownloadReasonMessages.update)
		default:
			return reason
	}
}

export function formatAnalyticsDependentProjectFallbackLabel(
	downloadReason: string | undefined,
	formatMessage: FormatMessage,
): string {
	const normalizedReason = downloadReason?.trim().toLowerCase()
	if (normalizedReason === 'standalone' || normalizedReason === 'update') {
		return formatMessage(analyticsMessages.noDependent)
	}

	return formatMessage(analyticsMessages.unknown)
}

export function formatAnalyticsDownloadSourceLabel(
	source: string,
	formatMessage: FormatMessage,
): string {
	const normalized = source.trim()
	const normalizedLowercase = normalized.toLowerCase()
	if (normalizedLowercase === 'website') {
		return formatMessage(analyticsDownloadSourceMessages.website)
	}
	if (normalizedLowercase === 'modrinth_app') {
		return formatMessage(analyticsDownloadSourceMessages.app)
	}
	if (!normalized.includes('_')) {
		return normalized
	}

	return normalizedLowercase
		.split('_')
		.filter((part) => part.length > 0)
		.map((part) => `${part.charAt(0).toUpperCase()}${part.slice(1)}`)
		.join(' ')
}

export function formatAnalyticsProjectStatusLabel(
	status: string,
	formatMessage: FormatMessage,
): string {
	switch (status.trim().toLowerCase()) {
		case 'approved':
			return formatMessage(analyticsProjectStatusMessages.approved)
		case 'archived':
			return formatMessage(analyticsProjectStatusMessages.archived)
		case 'rejected':
			return formatMessage(analyticsProjectStatusMessages.rejected)
		case 'draft':
			return formatMessage(analyticsProjectStatusMessages.draft)
		case 'unlisted':
			return formatMessage(analyticsProjectStatusMessages.unlisted)
		case 'withheld':
			return formatMessage(analyticsProjectStatusMessages.withheld)
		case 'private':
			return formatMessage(analyticsProjectStatusMessages.private)
		case 'other':
			return formatMessage(analyticsProjectStatusMessages.other)
		default:
			return capitalizeAnalyticsValue(status)
	}
}

export function formatAnalyticsLoaderLabel(loader: string, formatMessage: FormatMessage): string {
	const normalizedLoader = loader.trim()
	const loaderMessage = getLoaderMessage(normalizedLoader)
	return loaderMessage ? formatMessage(loaderMessage) : capitalizeAnalyticsValue(normalizedLoader)
}

function capitalizeAnalyticsValue(value: string): string {
	const normalizedValue = value.trim()
	if (normalizedValue.length === 0) {
		return value
	}

	return `${normalizedValue.charAt(0).toUpperCase()}${normalizedValue.slice(1)}`
}
