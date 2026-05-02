import { injectI18n, useDebugLogger } from '@modrinth/ui'
import dayjs from 'dayjs'
import { computed, ref, watch } from 'vue'

// note: build step can miss unix import for some reason, so
// we have to import it like this

const { unix } = dayjs

export function useCountryNames(style = 'long') {
	const { locale } = injectI18n()
	const displayNames = computed(
		() => new Intl.DisplayNames([locale.value], { type: 'region', style }),
	)
	return function formatCountryName(code) {
		try {
			return displayNames.value.of(code) ?? code
		} catch {
			return code
		}
	}
}

export const countryCodeToName = (code) => {
	const formatCountryName = useCountryNames()

	return formatCountryName(code)
}

export const countryCodeToFlag = (code) => {
	if (code === 'XX') {
		return undefined
	}
	return `https://flagcdn.com/h240/${code.toLowerCase()}.png`
}

export const formatTimestamp = (timestamp) => {
	return unix(timestamp).format()
}

export const formatPercent = (value, sum) => {
	return `${((value / sum) * 100).toFixed(2)}%`
}

const hashProjectId = (projectId) => {
	return projectId.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) % 30
}

export const defaultColors = [
	'#ff496e', // Original: Bright pink
	'#ffa347', // Original: Bright orange
	'#1bd96a', // Original: Bright green
	'#4f9cff', // Original: Bright blue
	'#c78aff', // Original: Bright purple
	'#ffeb3b', // Added: Bright yellow
	'#00bcd4', // Added: Bright cyan
	'#ff5722', // Added: Bright red-orange
	'#9c27b0', // Added: Bright deep purple
	'#3f51b5', // Added: Bright indigo
	'#009688', // Added: Bright teal
	'#cddc39', // Added: Bright lime
	'#795548', // Added: Bright brown
	'#607d8b', // Added: Bright blue-grey
]

/**
 * @param {string | number} value
 * @returns {string} color
 */
export const getDefaultColor = (value) => {
	if (typeof value === 'string') {
		value = hashProjectId(value)
	}
	return defaultColors[value % defaultColors.length]
}

export const intToRgba = (color, projectId = 'Unknown', theme = 'dark', alpha = '1') => {
	const hash = hashProjectId(projectId)

	if (!color || color === 0) {
		return getDefaultColor(hash)
	}

	// if color is a string, return that instead
	if (typeof color === 'string') {
		return color
	}

	// Extract RGB values
	let r = (color >> 16) & 255
	let g = (color >> 8) & 255
	let b = color & 255

	// Hash function to alter color slightly based on project_id
	r = (r + hash) % 256
	g = (g + hash) % 256
	b = (b + hash) % 256

	// Adjust brightness for theme
	const brightness = r * 0.299 + g * 0.587 + b * 0.114
	const threshold = theme === 'dark' ? 50 : 200
	if (theme === 'dark' && brightness < threshold) {
		// Increase brightness for dark theme
		r += threshold / 2
		g += threshold / 2
		b += threshold / 2
	} else if (theme === 'light' && brightness > threshold) {
		// Decrease brightness for light theme
		r -= threshold / 4
		g -= threshold / 4
		b -= threshold / 4
	}

	// Ensure RGB values are within 0-255
	r = Math.min(255, Math.max(0, r))
	g = Math.min(255, Math.max(0, g))
	b = Math.min(255, Math.max(0, b))

	return `rgba(${r}, ${g}, ${b}, ${alpha})`
}

const emptyAnalytics = {
	sum: 0,
	len: 0,
	chart: {
		labels: [],
		data: [],
		sumData: [
			{
				name: '',
				data: [],
			},
		],
		colors: [],
		defaultColors: [],
	},
	projectIds: [],
}

export const analyticsSetToCSVString = (analytics) => {
	if (!analytics) {
		return ''
	}

	const newline = '\n'
	const labels = analytics.chart.labels
	const projects = analytics.chart.data

	const projectNames = projects.map((p) => p.name)

	const header = ['Date', ...projectNames].join(',')

	const data = labels.map((label, i) => {
		const values = projects.map((p) => p.data?.[i] || '')
		return [label, ...values].join(',')
	})

	return [header, ...data].join(newline)
}

export const processAnalytics = (category, projects, labelFn, sortFn, mapFn, chartName, theme, startDate, endDate, timeResolution) => {
	if (!category || !projects) {
		return emptyAnalytics
	}

	// Get an intersection of category keys and project ids
	const projectIds = projects.map((p) => p.id)
	const loadedProjectIds = Object.keys(category).filter((id) => projectIds.includes(id))

	if (!loadedProjectIds?.length) {
		return emptyAnalytics
	}

	const loadedProjectData = loadedProjectIds.map((id) => category[id])

	// Convert each project's data into a list of [unix_ts_str, number] pairs
	const projectData = loadedProjectData
		.map((data) => Object.entries(data))
		.map((data) => data.sort(sortFn))
		.map((data) => (mapFn ? data.map(mapFn) : data))

	// Generate all expected timestamps for the date range and resolution
	// Match backend logic: round start down to resolution boundary, round end up
	let timestamps = []
	let aggregateFn = null

	if (startDate && endDate && timeResolution) {
		const stepSeconds = timeResolution * 60 // convert minutes to seconds
		const startTs = startDate.unix()
		const endTs = endDate.unix()

		// For "All time" (start near unix 0) or very large ranges, use actual data timestamps
		const durationDays = (endTs - startTs) / 86400
		const isAllTime = startTs < 86400 * 365 // Started before 1971

		if (isAllTime || durationDays > 365 * 5) {
			// Use union of actual data timestamps instead of generating all timestamps
			timestamps = Array.from(
				new Set(projectData.flatMap((data) => data.map(([ts]) => ts))),
			).sort()
		} else {
			// Round start_date down to nearest resolution (like backend does)
			const diff = startTs % stepSeconds
			const alignedStart = startTs - diff
			// Round end_date up to nearest resolution (like backend does)
			const endDiff = endTs % stepSeconds
			const alignedEnd = endTs + (stepSeconds - endDiff)

			// For large time ranges, aggregate by week or month
			if (durationDays > 365) {
				// Aggregate by month
				aggregateFn = (ts) => {
					const d = dayjs.unix(parseInt(ts))
					return d.startOf('month').unix().toString()
				}
			} else if (durationDays > 90) {
				// Aggregate by week
				aggregateFn = (ts) => {
					const d = dayjs.unix(parseInt(ts))
					return d.startOf('week').unix().toString()
				}
			}

			const expected = []
			let currentTs = alignedStart
			while (currentTs <= alignedEnd) {
				expected.push(currentTs.toString())
				currentTs += stepSeconds
			}
			timestamps = expected
		}
	} else {
		// Fallback to union of project timestamps if date range is not provided
		timestamps = Array.from(
			new Set(projectData.flatMap((data) => data.map(([ts]) => ts))),
		).sort()
	}

	// If we need to aggregate, process the data
	let finalProjectData = projectData
	let finalTimestamps = timestamps

	if (aggregateFn) {
		// Aggregate timestamps
		finalTimestamps = [...new Set(timestamps.map((ts) => aggregateFn(ts)))].sort()
		// Aggregate project data
		finalProjectData = projectData.map((data) => {
			const aggregated = {}
			data.forEach(([ts, val]) => {
				const key = aggregateFn(ts)
				aggregated[key] = (aggregated[key] || 0) + val
			})
			return Object.entries(aggregated).sort(sortFn)
		})
	}

	const chartData = finalProjectData
		.map((data, i) => {
			const project = projects.find((p) => p.id === loadedProjectIds[i])
			if (!project) {
				throw new Error(`Project ${loadedProjectIds[i]} not found`)
			}

			return {
				name: `${project.title}`,
				data: finalTimestamps.map((ts) => {
					const entry = data.find(([ets]) => ets === ts)
					return entry ? entry[1] : null
				}),
				id: project.id,
				color: project.color,
			}
		})
		.filter((project) => {
			// Filter out projects with no data at all (all nulls/zeros)
			return project.data.some((val) => val !== null && val > 0)
		})
		.sort(
			(a, b) =>
				b.data.reduce((acc, cur) => (cur !== null ? acc + cur : acc), 0) -
				a.data.reduce((acc, cur) => (cur !== null ? acc + cur : acc), 0),
		)

	const projectIdsSortedBySum = chartData.map((p) => p.id)

	return {
		// The total count of all the values across all projects
		sum: finalProjectData.reduce((acc, cur) => acc + cur.reduce((a, c) => a + c[1], 0), 0),
		len: finalTimestamps.length,
		chart: {
			labels: finalTimestamps.map(labelFn),
			data: chartData.map((x) => ({ name: x.name, data: x.data })),
		sumData: [
			{
				name: chartName,
				data: finalTimestamps.map((ts) => {
					const entries = finalProjectData.flat().filter(([ets]) => ets === ts)
					if (entries.length === 0) return null
					return entries.reduce((acc, cur) => acc + cur[1], 0)
				}),
			},
		],
			colors: finalProjectData.map((_, i) => {
				const project = chartData[i]

				return intToRgba(project.color, project.id, theme)
			}),
			defaultColors: finalProjectData.map((_, i) => {
				const project = chartData[i]
				return getDefaultColor(project.id)
			}),
		},
		projectIds: projectIdsSortedBySum,
	}
}

export const processAnalyticsByCountry = (category, projects, sortFn) => {
	if (!category || !projects) {
		return {
			sum: 0,
			len: 0,
			data: [],
		}
	}

	// Get an intersection of category keys and project ids
	const projectIds = projects.map((p) => p.id)
	const loadedProjectIds = Object.keys(category).filter((id) => projectIds.includes(id))

	if (!loadedProjectIds?.length) {
		return {
			sum: 0,
			len: 0,
			data: [],
		}
	}

	const loadedProjectData = loadedProjectIds.map((id) => category[id])

	// Convert each project's data into a list of [countrycode, number] pairs
	// Fold into a single list with summed values for each country over all projects

	const countrySums = new Map()

	loadedProjectData.forEach((data) => {
		Object.entries(data).forEach(([country, value]) => {
			const countryCode = country || 'XX'
			const current = countrySums.get(countryCode) || 0
			countrySums.set(countryCode, current + value)
		})
	})

	const entries = Array.from(countrySums.entries())

	return {
		sum: entries.reduce((acc, cur) => acc + cur[1], 0),
		len: entries.length,
		data: entries.sort(sortFn),
	}
}

const sortCount = ([, a], [, b]) => b - a
const sortTimestamp = ([a], [b]) => a - b
const roundValue = ([ts, value]) => [ts, Math.round(parseFloat(value) * 1000000) / 1000000]

const processCountryAnalytics = (c, projects) => processAnalyticsByCountry(c, projects, sortCount)
const processNumberAnalytics = (c, projects, theme, startDate, endDate, timeResolution) =>
	processAnalytics(c, projects, formatTimestamp, sortTimestamp, null, 'Downloads', theme, startDate, endDate, timeResolution)
const processRevAnalytics = (c, projects, theme, startDate, endDate, timeResolution) =>
	processAnalytics(c, projects, formatTimestamp, sortTimestamp, roundValue, 'Revenue', theme, startDate, endDate, timeResolution)

const useFetchAnalytics = (
	url,
	baseOptions = {
		apiVersion: 3,
	},
) => {
	return useBaseFetch(url, baseOptions)
}

/**
 * @param {Ref<any[]>} projects
 * @param {undefined | () => any} onDataRefresh
 */
export const useFetchAllAnalytics = (
	onDataRefresh,
	projects,
	selectedProjects,
	personalRevenue = false,
	startDate = ref(dayjs().subtract(30, 'days')),
	endDate = ref(dayjs()),
	timeResolution = ref(1440),
) => {
	const debug = useDebugLogger('useFetchAllAnalytics')
	debug('init', {
		projectCount: projects.value?.length,
		personalRevenue,
		startDate: startDate.value?.toISOString(),
		endDate: endDate.value?.toISOString(),
	})

	const downloadData = ref(null)
	const viewData = ref(null)
	const revenueData = ref(null)
	const downloadsByCountry = ref(null)
	const viewsByCountry = ref(null)
	const loading = ref(true)
	const error = ref(null)

	const formattedData = computed(() => ({
		downloads: processNumberAnalytics(downloadData.value, selectedProjects.value, undefined, startDate.value, endDate.value, timeResolution.value),
		views: processNumberAnalytics(viewData.value, selectedProjects.value, undefined, startDate.value, endDate.value, timeResolution.value),
		revenue: processRevAnalytics(revenueData.value, selectedProjects.value, theme.active, startDate.value, endDate.value, timeResolution.value),
		downloadsByCountry: processCountryAnalytics(downloadsByCountry.value, selectedProjects.value),
		viewsByCountry: processCountryAnalytics(viewsByCountry.value, selectedProjects.value),
	}))

	const theme = useTheme()

	const totalData = computed(() => ({
		downloads: processNumberAnalytics(downloadData.value, projects.value, theme.active, startDate.value, endDate.value, timeResolution.value),
		views: processNumberAnalytics(viewData.value, projects.value, theme.active, startDate.value, endDate.value, timeResolution.value),
		revenue: processRevAnalytics(revenueData.value, projects.value, theme.active, startDate.value, endDate.value, timeResolution.value),
	}))

	const buildQuery = () => {
		const q = {
			start_date: startDate.value.toISOString(),
			end_date: endDate.value.toISOString(),
			resolution_minutes: timeResolution.value,
		}

		if (projects.value?.length) {
			q.project_ids = JSON.stringify(projects.value.map((p) => p.id))
		}

		return q
	}

	const fetchData = async (query) => {
		debug('fetchData called', { query })
		const normalQuery = new URLSearchParams(query)
		const revenueQuery = new URLSearchParams(query)

		if (personalRevenue) {
			revenueQuery.delete('project_ids')
		}

		const qs = normalQuery.toString()
		const revenueQs = revenueQuery.toString()

		try {
			loading.value = true
			error.value = null

			debug('fetching all 5 endpoints...')
			const responses = await Promise.all([
				useFetchAnalytics(`analytics/downloads?${qs}`),
				useFetchAnalytics(`analytics/views?${qs}`),
				useFetchAnalytics(`analytics/revenue?${revenueQs}`),
				useFetchAnalytics(`analytics/countries/downloads?${qs}`),
				useFetchAnalytics(`analytics/countries/views?${qs}`),
			])
			debug('all 5 endpoints resolved', {
				downloads: Object.keys(responses[0] || {}).length,
				views: Object.keys(responses[1] || {}).length,
				revenue: Object.keys(responses[2] || {}).length,
			})

			const projectIds = new Set()
			if (projects.value) {
				projects.value.forEach((p) => projectIds.add(p.id))
			} else {
				Object.keys(responses[0] || {}).forEach((id) => projectIds.add(id))
			}

			debug('filtering to projectIds', { count: projectIds.size })

			const filterProjectIds = (data) => {
				const filtered = {}
				Object.entries(data).forEach(([id, values]) => {
					if (projectIds.has(id)) {
						filtered[id] = values
					}
				})
				return filtered
			}

			downloadData.value = filterProjectIds(responses[0] || {})
			viewData.value = filterProjectIds(responses[1] || {})
			revenueData.value = filterProjectIds(responses[2] || {})

			downloadsByCountry.value = responses[3] || {}
			viewsByCountry.value = responses[4] || {}
		} catch (e) {
			debug('fetchData error', e)
			error.value = e
		} finally {
			loading.value = false
			debug('fetchData done, loading=false')
		}
	}

	const fetch = async () => {
		debug('fetch() called', { projectCount: projects.value?.length })
		await fetchData(buildQuery())
		if (onDataRefresh) {
			onDataRefresh()
		}
	}

	watch(
		[() => startDate.value, () => endDate.value, () => timeResolution.value, () => projects.value],
		(newVals, oldVals) => {
			debug('watch triggered', { new: newVals, old: oldVals })
			fetch()
		},
	)

	const validProjectIds = computed(() => {
		const ids = new Set()

		if (downloadData.value) {
			Object.keys(downloadData.value).forEach((id) => ids.add(id))
		}

		if (viewData.value) {
			Object.keys(viewData.value).forEach((id) => ids.add(id))
		}

		if (revenueData.value) {
			// revenue will always have all project ids, but the ids may have an empty object or a ton of keys below a cent (0.00...) as values. We want to filter those out
			Object.entries(revenueData.value).forEach(([id, data]) => {
				if (Object.keys(data).length) {
					if (Object.values(data).some((v) => v >= 0.01)) {
						ids.add(id)
					}
				}
			})
		}

		return Array.from(ids)
	})

	return {
		// Configuration
		timeResolution,

		startDate,
		endDate,

		// Data
		downloadData,
		viewData,
		revenueData,
		downloadsByCountry,
		viewsByCountry,

		// Computed state
		validProjectIds,
		formattedData,
		totalData,
		loading,
		error,
		fetch,
	}
}
