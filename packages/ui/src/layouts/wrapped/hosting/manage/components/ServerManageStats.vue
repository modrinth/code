<template>
	<div
		data-pyro-server-stats
		style="font-variant-numeric: tabular-nums"
		class="flex select-none flex-col items-center gap-3 md:flex-row"
		:class="{ 'pointer-events-none': loading }"
		:aria-hidden="loading"
	>
		<component
			:is="metric.link ? RouterLink : 'div'"
			v-for="(metric, index) in metrics"
			:key="index"
			:to="metric.link && !loading ? metric.link : undefined"
			class="relative isolate min-h-[145px] w-full overflow-hidden rounded-[20px] bg-surface-3 p-5"
			:class="
				metric.link && !loading
					? 'cursor-pointer transition-transform duration-100 hover:brightness-125 active:scale-95'
					: ''
			"
		>
			<div class="relative z-10 flex flex-col gap-2">
				<div class="flex items-center justify-between">
					<span class="stat-drop-shadow flex items-center gap-2 font-medium text-lg text-primary">
						{{ metric.title }}
					</span>
					<span class="relative">
						<component :is="metric.icon" class="stat-drop-shadow relative z-10 size-8" />
						<!-- <div
							class="absolute -right-4 -top-4 -z-10 size-14 rounded-full bg-surface-3 opacity-50 blur-lg"
						/> -->
					</span>
				</div>
				<span class="stat-drop-shadow text-4xl font-bold text-contrast">
					{{ metric.value
					}}<span
						v-if="metric.secondary"
						class="ml-1 text-sm font-normal stat-drop-shadow text-secondary"
						>{{ metric.secondary }}</span
					>
				</span>
				<!-- <div
					class="absolute -left-8 -top-4 -z-10 h-28 w-56 rounded-full bg-surface-3 opacity-50 blur-lg"
				/> -->
			</div>

			<div v-if="metric.showGraph" class="chart-space absolute bottom-0 left-0 right-0">
				<VueApexCharts
					v-if="isClient && !loading && metric.chartOptions"
					type="area"
					height="142"
					:options="metric.chartOptions"
					:series="metric.series!"
					class="chart"
					:class="chartsReady.has(index) ? 'opacity-100' : 'opacity-0'"
				/>
			</div>
		</component>
	</div>
</template>

<script setup lang="ts">
import { CpuIcon, DatabaseIcon, FolderOpenIcon } from '@modrinth/assets'
import type { Stats } from '@modrinth/utils'
import { useStorage } from '@vueuse/core'
import { computed, defineAsyncComponent, onMounted, ref, shallowRef, watch } from 'vue'
import { RouterLink } from 'vue-router'

import { injectModrinthServerContext, injectPageContext } from '#ui/providers'

const VueApexCharts = defineAsyncComponent(() => import('vue3-apexcharts'))

// apexcharts touches `window` at module load time, so we must not let SSR
// resolve the async component. Render only after mount on the client.
const isClient = ref(false)
onMounted(() => {
	isClient.value = true
})

const { serverId } = injectModrinthServerContext()
const { featureFlags } = injectPageContext()

const props = withDefaults(
	defineProps<{
		data?: Stats
		loading?: boolean
		showMemoryAsBytes?: boolean
	}>(),
	{
		data: undefined,
		loading: false,
		showMemoryAsBytes: false,
	},
)

const chartsReady = ref(new Set<number>())
const userPreferences = useStorage(`pyro-server-${serverId || 'unknown'}-preferences`, {
	ramAsNumber: false,
})
const isRamAsBytesForcedByFeatureFlag = computed(
	() => featureFlags?.serverRamAsBytesAlwaysOn?.value ?? false,
)

const showRamAsBytes = computed(
	() =>
		props.showMemoryAsBytes ||
		isRamAsBytesForcedByFeatureFlag.value ||
		userPreferences.value.ramAsNumber,
)

const stats = shallowRef(
	props.data?.current || {
		cpu_percent: 0,
		ram_usage_bytes: 0,
		ram_total_bytes: 1,
		storage_usage_bytes: 0,
	},
)

const GRAPH_SIZE = 10

const padGraph = (data: number[]) => {
	const capped = data.map((v) => Math.min(v, 100))
	if (capped.length >= GRAPH_SIZE) return capped.slice(-GRAPH_SIZE)
	return [...Array(GRAPH_SIZE - capped.length).fill(0), ...capped]
}

const cpuData = computed(() => padGraph(props.data?.graph.cpu ?? []))
const ramData = computed(() => padGraph(props.data?.graph.ram ?? []))

const cpuPercent = computed(() => stats.value.cpu_percent ?? 0)
const ramPercent = computed(
	() => ((stats.value.ram_usage_bytes ?? 0) / (stats.value.ram_total_bytes || 1)) * 100,
)

const cpuWarning = computed(() => cpuPercent.value >= 90)
const ramWarning = computed(() => ramPercent.value >= 90)

const cpuDataMax = 104
const ramDataMax = 104

const onChartReady = (index: number) => {
	chartsReady.value.add(index)
}

const buildChartOptions = (warning: boolean, index: number, dataMax: number) => ({
	chart: {
		type: 'area' as const,
		animations: { enabled: false },
		sparkline: { enabled: true },
		toolbar: { show: false },
		padding: { left: -10, right: -10, top: 0, bottom: 0 },
		events: {
			mounted: () => onChartReady(index),
			updated: () => onChartReady(index),
		},
	},
	stroke: { curve: 'smooth' as const, width: 3 },
	fill: {
		type: 'gradient' as const,
		gradient: { shadeIntensity: 1, opacityFrom: 0.25, opacityTo: 0.05, stops: [0, 100] },
	},
	tooltip: { enabled: false },
	grid: { show: false },
	xaxis: {
		labels: { show: false },
		axisBorder: { show: false },
		type: 'numeric' as const,
		tickAmount: GRAPH_SIZE,
	},
	yaxis: { show: false, min: 0, max: dataMax, forceNiceScale: false },
	colors: [warning ? 'var(--color-orange)' : 'var(--color-brand)'],
	dataLabels: { enabled: false },
})

const cpuChartOptions = computed(() => buildChartOptions(cpuWarning.value, 0, cpuDataMax))
const ramChartOptions = computed(() => buildChartOptions(ramWarning.value, 1, ramDataMax))

const cpuSeries = computed(() => [{ name: 'CPU', data: cpuData.value }])
const ramSeries = computed(() => [{ name: 'Memory', data: ramData.value }])

const formatBytes = (bytes: number) => {
	const units = ['B', 'KB', 'MB', 'GB']
	let value = bytes
	let unit = 0
	while (value >= 1024 && unit < units.length - 1) {
		value /= 1024
		unit++
	}
	return `${Math.round(value * 10) / 10} ${units[unit]}`
}

const metrics = computed(() => {
	const storageMetric = {
		title: 'Storage',
		value: props.loading ? '0 B' : formatBytes(stats.value.storage_usage_bytes ?? 0),
		secondary: null as string | null,
		icon: FolderOpenIcon,
		showGraph: false,
		chartOptions: null as ReturnType<typeof buildChartOptions> | null,
		series: null as { name: string; data: number[] }[] | null,
		link: `/hosting/manage/${encodeURIComponent(serverId)}/files`,
	}

	if (props.loading) {
		return [
			{
				title: 'CPU',
				value: '0.00%',
				secondary: null as string | null,
				icon: CpuIcon,
				showGraph: true,
				chartOptions: cpuChartOptions.value,
				series: cpuSeries.value,
				link: null,
			},
			{
				title: 'Memory',
				value: '0.00%',
				secondary: null as string | null,
				icon: DatabaseIcon,
				showGraph: true,
				chartOptions: ramChartOptions.value,
				series: ramSeries.value,
				link: null,
			},
			storageMetric,
		]
	}

	return [
		{
			title: 'CPU',
			value: `${cpuPercent.value.toFixed(2)}%`,
			secondary: null as string | null,
			icon: CpuIcon,
			showGraph: true,
			chartOptions: cpuChartOptions.value,
			series: cpuSeries.value,
			link: null,
		},
		{
			title: 'Memory',
			value: showRamAsBytes.value
				? formatBytes(stats.value.ram_usage_bytes ?? 0)
				: `${ramPercent.value.toFixed(2)}%`,
			secondary: showRamAsBytes.value
				? `/ ${formatBytes(stats.value.ram_total_bytes ?? 0)}`
				: (null as string | null),
			icon: DatabaseIcon,
			showGraph: true,
			chartOptions: ramChartOptions.value,
			series: ramSeries.value,
			link: null,
		},
		storageMetric,
	]
})

watch(
	() => props.data?.current,
	(newStats) => {
		if (newStats) {
			stats.value = newStats
		}
	},
)
</script>

<style scoped>
.stat-drop-shadow {
	filter: drop-shadow(0 4px 6px var(--surface-3));
}

.chart-space {
	height: 142px;
	width: calc(100% + 40px);
	margin-left: -20px;
	margin-right: -20px;
}

.chart {
	width: 100% !important;
	height: 142px !important;
	transition: opacity 0.3s ease-out;
	box-shadow:
		0 1px 2px 0 rgba(0, 0, 0, 0.3),
		0 1px 3px 0 rgba(0, 0, 0, 0.15);
}

.chart :deep(svg) {
	overflow: visible;
}
</style>
