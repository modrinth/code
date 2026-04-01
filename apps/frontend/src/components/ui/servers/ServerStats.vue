<template>
	<div
		data-pyro-server-stats
		style="font-variant-numeric: tabular-nums"
		class="flex select-none flex-col items-center gap-6 md:flex-row"
		:class="{ 'pointer-events-none': loading }"
		:aria-hidden="loading"
	>
		<component
			:is="metric.link ? NuxtLink : 'div'"
			v-for="(metric, index) in metrics"
			:key="index"
			:to="metric.link && !loading ? metric.link : undefined"
			class="relative isolate min-h-[145px] w-full overflow-hidden rounded-[20px] bg-bg-raised p-5"
			:class="
				metric.link && !loading
					? 'cursor-pointer transition-transform duration-100 hover:scale-105 active:scale-100'
					: ''
			"
		>
			<div class="relative z-10 flex flex-col gap-3">
				<div class="flex items-center justify-between">
					<span class="flex items-center gap-2 text-base font-medium text-primary">
						{{ metric.title }}
						<IssuesIcon
							v-if="metric.warning && !loading"
							v-tooltip="metric.warning"
							class="size-5"
							:style="{ color: 'var(--color-orange)' }"
						/>
					</span>
					<component :is="metric.icon" class="size-7" />
				</div>
				<span class="text-4xl font-semibold text-contrast">
					{{ metric.value }}
				</span>
			</div>

			<div v-if="metric.showGraph" class="chart-space absolute bottom-0 left-0 right-0">
				<ClientOnly>
					<VueApexCharts
						v-if="!loading"
						type="area"
						height="142"
						:options="getChartOptions(metric.warning, index)"
						:series="[{ name: metric.title, data: metric.data }]"
						class="chart"
						:class="chartsReady.has(index) ? 'opacity-100' : 'opacity-0'"
					/>
				</ClientOnly>
			</div>
		</component>
	</div>
</template>

<script setup lang="ts">
import { NuxtLink } from '#components'
import { CpuIcon, DatabaseIcon, FolderOpenIcon, IssuesIcon } from '@modrinth/assets'
import type { Stats } from '@modrinth/utils'
import { useStorage } from '@vueuse/core'
import { computed, ref, shallowRef } from 'vue'

const flags = useFeatureFlags()
const route = useNativeRoute()
const serverId = route.params.id
const VueApexCharts = defineAsyncComponent(() => import('vue3-apexcharts'))

const chartsReady = ref(new Set<number>())

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
	ramAsNumber: false,
})

const props = withDefaults(defineProps<{ data?: Stats; loading?: boolean }>(), {
	loading: false,
})

const stats = shallowRef(
	props.data?.current || {
		cpu_percent: 0,
		ram_usage_bytes: 0,
		ram_total_bytes: 1, // Avoid division by zero
		storage_usage_bytes: 0,
	},
)

const onChartReady = (index: number) => {
	chartsReady.value.add(index)
}

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

const cpuData = ref<number[]>(Array(20).fill(0))
const ramData = ref<number[]>(Array(20).fill(0))

const updateGraphData = (arr: number[], newValue: number) => {
	arr.push(newValue)
	arr.shift()
}

const metrics = computed(() => {
	const storageMetric = {
		title: 'Storage',
		value: props.loading ? '0 B' : formatBytes(stats.value.storage_usage_bytes),
		icon: FolderOpenIcon,
		data: [] as number[],
		showGraph: false,
		warning: null,
		link: `/hosting/manage/${serverId}/files`,
	}

	if (props.loading) {
		return [
			{
				title: 'CPU',
				value: '0.00%',
				icon: CpuIcon,
				data: cpuData.value,
				showGraph: false,
				warning: null,
				link: null,
			},
			{
				title: 'Memory',
				value: '0.00%',
				icon: DatabaseIcon,
				data: ramData.value,
				showGraph: false,
				warning: null,
				link: null,
			},
			storageMetric,
		]
	}

	const ramPercent = Math.min(
		(stats.value.ram_usage_bytes / stats.value.ram_total_bytes) * 100,
		100,
	)
	const cpuPercent = Math.min(stats.value.cpu_percent, 100)

	updateGraphData(cpuData.value, cpuPercent)
	updateGraphData(ramData.value, ramPercent)

	return [
		{
			title: 'CPU',
			value: `${cpuPercent.toFixed(2)}%`,
			icon: CpuIcon,
			data: cpuData.value,
			showGraph: true,
			warning: cpuPercent >= 90 ? 'CPU usage is very high' : null,
			link: null,
		},
		{
			title: 'Memory',
			value:
				userPreferences.value.ramAsNumber || flags.value.developerMode
					? formatBytes(stats.value.ram_usage_bytes)
					: `${ramPercent.toFixed(2)}%`,
			icon: DatabaseIcon,
			data: ramData.value,
			showGraph: true,
			warning: ramPercent >= 90 ? 'Memory usage is very high' : null,
			link: null,
		},
		storageMetric,
	]
})

const getChartOptions = (hasWarning: string | null, index: number) => ({
	chart: {
		type: 'area' as const,
		animations: { enabled: false },
		sparkline: { enabled: true },
		toolbar: { show: false },
		padding: {
			left: -10,
			right: -10,
			top: 0,
			bottom: 0,
		},
		events: {
			mounted: () => onChartReady(index),
			updated: () => onChartReady(index),
		},
	},
	stroke: { curve: 'smooth', width: 3 },
	fill: {
		type: 'gradient',
		gradient: {
			shadeIntensity: 1,
			opacityFrom: 0.25,
			opacityTo: 0.05,
			stops: [0, 100],
		},
	},
	tooltip: { enabled: false },
	grid: { show: false },
	xaxis: {
		labels: { show: false },
		axisBorder: { show: false },
		type: 'numeric',
		tickAmount: 20,
		range: 20,
	},
	yaxis: {
		show: false,
		min: 0,
		max: 100,
		forceNiceScale: false,
	},
	colors: [hasWarning ? 'var(--color-orange)' : 'var(--color-brand)'],
	dataLabels: {
		enabled: false,
	},
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
}
</style>
