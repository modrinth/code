<template>
  <div
    data-pyro-server-stats
    style="font-variant-numeric: tabular-nums"
    class="flex select-none flex-col items-center gap-6 md:flex-row"
    :class="{ 'pointer-events-none': loading }"
    :aria-hidden="loading"
  >
    <div
      v-for="(metric, index) in metrics"
      :key="index"
      class="relative isolate min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
    >
      <div class="relative z-10 -ml-3 w-fit rounded-xl px-3 py-1">
        <div class="relative z-10">
          <div class="-mb-0.5 mt-0.5 flex flex-row items-center gap-2">
            <h2 class="m-0 -ml-0.5 text-3xl font-extrabold text-contrast">{{ metric.value }}</h2>
            <h3 class="text-sm font-normal text-secondary">/ {{ metric.max }}</h3>
          </div>
          <h3 class="flex items-center gap-2 text-base font-normal text-secondary">
            {{ metric.title }}
            <IssuesIcon
              v-if="metric.warning && !loading"
              v-tooltip="metric.warning"
              class="size-5"
              :style="{ color: 'var(--color-orange)' }"
            />
          </h3>
        </div>
        <div class="absolute -left-8 -top-4 h-28 w-56 rounded-full bg-bg-raised blur-lg" />
      </div>

      <component
        :is="metric.icon"
        class="absolute right-10 top-10 z-10 size-8"
        style="width: 2rem; height: 2rem"
      />

      <div class="chart-space absolute bottom-0 left-0 right-0">
        <ClientOnly>
          <VueApexCharts
            v-if="metric.showGraph && !loading"
            type="area"
            height="142"
            :options="getChartOptions(metric.warning, index)"
            :series="[{ name: metric.title, data: metric.data }]"
            class="chart"
            :class="chartsReady.has(index) ? 'opacity-100' : 'opacity-0'"
          />
        </ClientOnly>
      </div>
    </div>
    <nuxt-link
      :to="loading ? undefined : `/servers/manage/${serverId}/files`"
      class="relative isolate min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
      :class="loading ? '' : 'transition-transform duration-100 hover:scale-105 active:scale-100'"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 -ml-0.5 mt-1 text-3xl font-extrabold text-contrast">
          {{ loading ? "0 B" : formatBytes(stats.storage_usage_bytes) }}
        </h2>
      </div>
      <h3 class="text-base font-normal text-secondary">Storage usage</h3>
      <FolderOpenIcon class="absolute right-10 top-10 size-8" />
    </nuxt-link>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, shallowRef } from "vue";
import { FolderOpenIcon, CPUIcon, DatabaseIcon, IssuesIcon } from "@modrinth/assets";
import { useStorage } from "@vueuse/core";
import type { Stats } from "@modrinth/utils";

const flags = useFeatureFlags();
const route = useNativeRoute();
const serverId = route.params.id;
const VueApexCharts = defineAsyncComponent(() => import("vue3-apexcharts"));

const chartsReady = ref(new Set<number>());

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  ramAsNumber: false,
});

const props = withDefaults(defineProps<{ data?: Stats; loading?: boolean }>(), {
  loading: false,
});

const stats = shallowRef(
  props.data?.current || {
    cpu_percent: 0,
    ram_usage_bytes: 0,
    ram_total_bytes: 1, // Avoid division by zero
    storage_usage_bytes: 0,
  },
);

const onChartReady = (index: number) => {
  chartsReady.value.add(index);
};

const formatBytes = (bytes: number) => {
  const units = ["B", "KB", "MB", "GB"];
  let value = bytes;
  let unit = 0;
  while (value >= 1024 && unit < units.length - 1) {
    value /= 1024;
    unit++;
  }
  return `${Math.round(value * 10) / 10} ${units[unit]}`;
};

const cpuData = ref<number[]>(Array(20).fill(0));
const ramData = ref<number[]>(Array(20).fill(0));

const updateGraphData = (arr: number[], newValue: number) => {
  arr.push(newValue);
  arr.shift();
};

const metrics = computed(() => {
  if (props.loading) {
    return [
      {
        title: "CPU usage",
        value: "0.00%",
        max: "100%",
        icon: CPUIcon,
        data: cpuData.value,
        showGraph: false,
        warning: null,
      },
      {
        title: "Memory usage",
        value: "0.00%",
        max: "100%",
        icon: DatabaseIcon,
        data: ramData.value,
        showGraph: false,
        warning: null,
      },
    ];
  }

  const ramPercent = Math.min(
    (stats.value.ram_usage_bytes / stats.value.ram_total_bytes) * 100,
    100,
  );
  const cpuPercent = Math.min(stats.value.cpu_percent, 100);

  updateGraphData(cpuData.value, cpuPercent);
  updateGraphData(ramData.value, ramPercent);

  return [
    {
      title: "CPU usage",
      value: `${cpuPercent.toFixed(2)}%`,
      max: "100%",
      icon: CPUIcon,
      data: cpuData.value,
      showGraph: true,
      warning: cpuPercent >= 90 ? "CPU usage is very high" : null,
    },
    {
      title: "Memory usage",
      value:
        userPreferences.value.ramAsNumber || flags.developerMode
          ? formatBytes(stats.value.ram_usage_bytes)
          : `${ramPercent.toFixed(2)}%`,
      max:
        userPreferences.value.ramAsNumber || flags.developerMode
          ? formatBytes(stats.value.ram_total_bytes)
          : "100%",
      icon: DatabaseIcon,
      data: ramData.value,
      showGraph: true,
      warning: ramPercent >= 90 ? "Memory usage is very high" : null,
    },
  ];
});

const getChartOptions = (hasWarning: string | null, index: number) => ({
  chart: {
    type: "area",
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
  stroke: { curve: "smooth", width: 3 },
  fill: {
    type: "gradient",
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
    type: "numeric",
    tickAmount: 20,
    range: 20,
  },
  yaxis: {
    show: false,
    min: 0,
    max: 100,
    forceNiceScale: false,
  },
  colors: [hasWarning ? "var(--color-orange)" : "var(--color-brand)"],
  dataLabels: {
    enabled: false,
  },
});

watch(
  () => props.data?.current,
  (newStats) => {
    if (newStats) {
      stats.value = newStats;
    }
  },
);
</script>

<style scoped>
.chart-space {
  height: 142px;
  width: calc(100% + 48px);
  margin-left: -24px;
  margin-right: -24px;
}

.chart {
  width: 100% !important;
  height: 142px !important;
  transition: opacity 0.3s ease-out;
}
</style>
