<template>
  <div data-pyro-server-stats class="flex select-none flex-col items-center gap-6 md:flex-row">
    <div
      v-for="(metric, index) in metrics"
      :key="index"
      class="relative min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
    >
      <div class="relative z-10 flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ metric.value }}
        </h2>
        <h3 class="relative z-10 text-sm font-normal text-secondary">/ {{ metric.max }}</h3>
      </div>
      <h3 class="relative z-10 text-base font-normal text-secondary">{{ metric.title }}</h3>

      <component :is="metric.icon" class="absolute right-8 top-8" />
      <ClientOnly>
        <VueApexCharts
          ref="chart"
          type="area"
          height="100"
          :options="chartOptions"
          :series="[{ name: 'Chart', data: metric.data }]"
          class="chart chart-animation absolute bottom-0 left-0 right-0 w-full"
        />
      </ClientOnly>
    </div>

    <NuxtLink
      :to="`/servers/manage/${serverId}/files`"
      class="relative min-h-[150px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8 transition-transform duration-100 hover:scale-105 active:scale-100"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ formatBytes(data.current.storage_usage_bytes) }}
        </h2>
        <h3 class="relative z-10 text-sm font-normal text-secondary">
          / {{ formatBytes(data.current.storage_total_bytes) }}
        </h3>
      </div>
      <h3 class="relative z-10 text-base font-normal text-secondary">Storage usage</h3>

      <FolderOpenIcon class="absolute right-8 top-8 size-8" />
    </NuxtLink>
  </div>
</template>

<script setup lang="ts">
import { FolderOpenIcon, CPUIcon, DBIcon } from "@modrinth/assets";
import { useStorage } from "@vueuse/core";
import type { Stats } from "~/types/servers";

const route = useNativeRoute();
const serverId = route.params.id;

const userPrefrences = useStorage(`pyro-server-${serverId}-preferences`, {
  ramAsNumber: false,
});

const VueApexCharts = defineAsyncComponent(() => import("vue3-apexcharts"));

const props = defineProps({
  data: {
    type: Object as PropType<Stats>,
    required: true,
  },
});

const lerp = (a: number, b: number) => {
  return a + (b - a) * 0.5;
};

// I told you it would go into prod
const formatBytes = (bytes: number) => {
  const units = ["Bytes", "KB", "MB", "GB", "TB"];
  let value = bytes;
  let unitIndex = 0;

  while (value >= 1024 && unitIndex < units.length - 2) {
    value /= 1024;
    unitIndex++;
  }

  return `${Math.round(value * 100) / 100} ${units[unitIndex]}`;
};

const metrics = ref([
  {
    title: "CPU usage",
    value: "0%",
    max: "100%",
    icon: markRaw(CPUIcon),
    data: [] as number[],
  },
  {
    title: "Memory usage",
    value: "0%",
    max: userPrefrences.value.ramAsNumber
      ? formatBytes(props.data.current.ram_total_bytes)
      : "100%",
    icon: markRaw(DBIcon),
    data: [] as number[],
  },
]);

const updateMetrics = () => {
  metrics.value = metrics.value.map((metric, index) => {
    if (userPrefrences.value.ramAsNumber && index === 1) {
      return {
        ...metric,
        value: formatBytes(props.data.current.ram_usage_bytes),
        data: [...metric.data.slice(-10), props.data.current.ram_usage_bytes],
      };
    } else {
      const currentValue =
        index === 0
          ? props.data.current.cpu_percent
          : Math.min(
              (props.data.current.ram_usage_bytes / props.data.current.ram_total_bytes) * 100,
              100,
            );
      const pastValue =
        index === 0
          ? props.data.past.cpu_percent
          : Math.min(
              (props.data.past.ram_usage_bytes / props.data.past.ram_total_bytes) * 100,
              100,
            );

      const newValue = lerp(currentValue, pastValue);
      return {
        ...metric,
        value: `${newValue.toFixed(2)}%`,
        data: [...metric.data.slice(-10), newValue],
      };
    }
  });
};

const chartOptions = ref({
  chart: {
    id: "stats",
    fontFamily:
      "Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Oxygen, Ubuntu, Roboto, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif",
    foreColor: "var(--color-base)",
    toolbar: { show: false },
    zoom: { enabled: false },
    sparkline: { enabled: true },
    animations: {
      enabled: true,
      easing: "linear",
      dynamicAnimation: { speed: 1000 },
    },
  },
  stroke: { curve: "smooth" },
  fill: {
    colors: ["var(--color-brand)"],
    type: "gradient",
    opacity: 1,
    gradient: {
      shade: "light",
      type: "vertical",
      shadeIntensity: 0,
      gradientToColors: ["var(--color-brand)"],
      inverseColors: true,
      opacityFrom: 0.5,
      opacityTo: 0,
      stops: [0, 100],
      colorStops: [],
    },
  },
  grid: { show: false },
  legend: { show: false },
  colors: ["var(--color-brand)"],
  dataLabels: { enabled: false },
  xaxis: {
    type: "numeric",
    lines: { show: false },
    axisBorder: { show: false },
    axisTicks: { show: false },
    labels: { show: false },
  },
  yaxis: {
    min: 0,
    max: 100,
    tickAmount: 5,
    labels: { show: false },
    axisBorder: { show: false },
    axisTicks: { show: false },
  },
  tooltip: { enabled: false },
});

let interval: number;

onMounted(() => {
  updateMetrics();
  interval = window.setInterval(updateMetrics, 1000);
});

onUnmounted(() => {
  if (interval) {
    clearInterval(interval);
  }
});
</script>

<style scoped>
@keyframes chart-enter-animation {
  0% {
    opacity: 0;
  }
  100% {
    opacity: 1;
  }
}

.chart-animation {
  opacity: 0;
  animation: chart-enter-animation 0.5s ease-out forwards;
  animation-delay: 1s;
}
</style>
