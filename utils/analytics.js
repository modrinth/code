import dayjs from 'dayjs'

// note: build step can miss unix import for some reason, so
// we have to import it like this
// eslint-disable-next-line import/no-named-as-default-member
const { unix } = dayjs

export function useCountryNames(style = 'long') {
  const formattingOptions = { type: 'region', style }
  const { formats } = useVIntl()
  return function formatCountryName(code) {
    return formats.displayName(code, formattingOptions)
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

const intToRgba = (color, projectId = 'Unknown', theme) => {
  // Extract RGB values
  let r = (color >> 16) & 255
  let g = (color >> 8) & 255
  let b = color & 255

  // Hash function to alter color slightly based on project_id
  const hash = projectId.split('').reduce((acc, char) => acc + char.charCodeAt(0), 0) % 30
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

  return `rgba(${r}, ${g}, ${b}, 1)`
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
  },
}

export const processAnalytics = (category, projects, labelFn, sortFn, mapFn, chartName) => {
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
  // Sort, label&map
  const projectData = loadedProjectData
    .map((data) => Object.entries(data))
    .map((data) => data.sort(sortFn))
    .map((data) => (mapFn ? data.map(mapFn) : data))

  // Each project may not include the same timestamps, so we should use the union of all timestamps
  const timestamps = Array.from(
    new Set(projectData.flatMap((data) => data.map(([ts]) => ts)))
  ).sort()

  const chartData = projectData
    .map((data, i) => {
      const project = projects.find((p) => p.id === loadedProjectIds[i])
      if (!project) {
        throw new Error(`Project ${loadedProjectIds[i]} not found`)
      }

      return {
        name: `${project.title}`,
        data: timestamps.map((ts) => {
          const entry = data.find(([ets]) => ets === ts)
          return entry ? entry[1] : 0
        }),
        id: project.id,
        color: project.color,
      }
    })
    .sort(
      (a, b) =>
        b.data.reduce((acc, cur) => acc + cur, 0) - a.data.reduce((acc, cur) => acc + cur, 0)
    )

  return {
    // The total count of all the values across all projects
    sum: projectData.reduce((acc, cur) => acc + cur.reduce((a, c) => a + c[1], 0), 0),
    len: timestamps.length,
    chart: {
      labels: timestamps.map(labelFn),
      data: chartData.map((x) => ({ name: x.name, data: x.data })),
      sumData: [
        {
          name: chartName,
          data: timestamps.map((ts) => {
            const entries = projectData.flat().filter(([ets]) => ets === ts)
            return entries.reduce((acc, cur) => acc + cur[1], 0)
          }),
        },
      ],
      colors: projectData.map((_, i) => {
        const theme = useTheme()
        const project = chartData[i]

        return project.color
          ? intToRgba(project.color, project.id, theme.value.value)
          : '--color-brand'
      }),
    },
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
      const current = countrySums.get(country) || 0
      countrySums.set(country, current + value)
    })
  })

  const entries = Array.from(countrySums.entries())

  return {
    sum: entries.reduce((acc, cur) => acc + cur[1], 0),
    len: entries.length,
    data: entries.sort(sortFn),
  }
}

const sortCount = ([_a, a], [_b, b]) => b - a
const sortTimestamp = ([a], [b]) => a - b
const roundValue = ([ts, value]) => [ts, Math.round(parseFloat(value) * 100) / 100]

const processCountryAnalytics = (c, projects) => processAnalyticsByCountry(c, projects, sortCount)
const processNumberAnalytics = (c, projects) =>
  processAnalytics(c, projects, formatTimestamp, sortTimestamp, null, 'Downloads')
const processRevAnalytics = (c, projects) =>
  processAnalytics(c, projects, formatTimestamp, sortTimestamp, roundValue, 'Revenue')

const useFetchAnalytics = (
  url,
  baseOptions = {
    apiVersion: 3,
  }
) => {
  return useBaseFetch(url, baseOptions)
}

/**
 * @param {any} projects
 * @param {undefined | () => any} onDataRefresh
 */
export const useFetchAllAnalytics = (onDataRefresh, projects = undefined) => {
  const timeResolution = ref(1440) // 1 day
  const timeRange = ref(43200) // 30 days

  const startDate = ref(Date.now() - timeRange.value * 60 * 1000)
  const endDate = ref(Date.now())

  const downloadData = ref(null)
  const viewData = ref(null)
  const revenueData = ref(null)
  const downloadsByCountry = ref(null)
  const viewsByCountry = ref(null)
  const loading = ref(true)
  const error = ref(null)

  const formattedData = computed(() => ({
    downloads: processNumberAnalytics(downloadData.value, projects),
    views: processNumberAnalytics(viewData.value, projects),
    revenue: processRevAnalytics(revenueData.value, projects),
    downloadsByCountry: processCountryAnalytics(downloadsByCountry.value, projects),
    viewsByCountry: processCountryAnalytics(viewsByCountry.value, projects),
  }))

  const fetchData = async (query) => {
    const normalQuery = new URLSearchParams(query)

    const revQuery = new URLSearchParams(query)

    const qs = normalQuery.toString()
    const revQs = revQuery.toString()

    try {
      loading.value = true
      error.value = null

      const responses = await Promise.all([
        useFetchAnalytics(`analytics/downloads?${qs}`),
        useFetchAnalytics(`analytics/views?${qs}`),
        useFetchAnalytics(`analytics/revenue?${revQs}`),
        useFetchAnalytics(`analytics/countries/downloads?${qs}`),
        useFetchAnalytics(`analytics/countries/views?${qs}`),
      ])

      downloadData.value = responses[0] || {}
      viewData.value = responses[1] || {}
      revenueData.value = responses[2] || {}
      downloadsByCountry.value = responses[3] || {}
      viewsByCountry.value = responses[4] || {}
    } catch (e) {
      error.value = e
    } finally {
      loading.value = false
    }
  }

  watch(
    [() => startDate.value, () => endDate.value, () => timeResolution.value, () => projects],
    async () => {
      const q = {
        start_date: dayjs(startDate.value).toISOString(),
        end_date: dayjs(endDate.value).toISOString(),
        resolution_minutes: timeResolution.value,
      }

      if (projects?.length) {
        q.project_ids = JSON.stringify(projects.map((p) => p.id))
      }

      await fetchData(q)

      if (onDataRefresh) {
        onDataRefresh()
      }
    },
    {
      immediate: true,
    }
  )

  return {
    // Configuration
    timeResolution,
    timeRange,

    startDate,
    endDate,

    // Data
    downloadData,
    viewData,
    revenueData,
    downloadsByCountry,
    viewsByCountry,

    // Computed state
    formattedData,
    loading,
    error,
  }
}
