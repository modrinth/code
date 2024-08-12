<template>
  <div data-pyro-server-stats class="flex flex-row items-center gap-6">
    <div
      v-for="(metric, index) in metrics"
      :key="index"
      class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ metric.value }}
        </h2>
        <ChevronRightIcon />
      </div>
      <h3>{{ metric.title }}</h3>

      <component class="absolute right-8 top-8" :is="metric.icon" />
      <ClientOnly>
        <VueApexCharts
          ref="chart"
          type="area"
          height="150"
          :options="chartOptions"
          :series="[{ name: 'Chart', data: metric.data }]"
          class="chart absolute bottom-0 left-0 right-0 w-full"
        />
      </ClientOnly>
    </div>

    <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ (Math.round((data.current.storage_total_bytes / 1e9) * 100) / 100).toFixed(2) }} GB
        </h2>
        <ChevronRightIcon />
      </div>
      <h3>Storage usage</h3>

      <div class="flex flex-col gap-2 pt-3">
        <div class="flex h-5 flex-row items-center gap-2 text-sm">
          <FolderOpenIcon />
          <p>World</p>
        </div>
        <NuxtLink
          :to="`/servers/manage/${serverId}/options/properties`"
          class="flex h-5 flex-row items-center gap-2 text-sm"
        >
          <FileTextIcon />
          <p>Server properties</p>
        </NuxtLink>
        <div class="flex h-5 flex-row items-center gap-2 text-sm">
          <FileTextIcon />
          <p>Paper configuration</p>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, onUnmounted, markRaw } from "vue";
import type { Stats } from "~/types/servers";
import { FileTextIcon, FolderOpenIcon, ChevronRightIcon, CPUIcon, DBIcon } from "@modrinth/assets";

const VueApexCharts = defineAsyncComponent(() => import("vue3-apexcharts"));

const route = useNativeRoute();
const serverId = route.params.id;

const props = defineProps({
  data: {
    type: Object as PropType<Stats>,
    required: true,
  },
});

const lerp = (a: number, b: number) => {
  return a + (b - a) * 0.5;
};

const metrics = ref([
  {
    title: "CPU usage",
    value: "0%",
    icon: markRaw(CPUIcon),
    data: [] as number[],
  },
  {
    title: "Memory usage",
    value: "0%",
    icon: markRaw(DBIcon),
    data: [] as number[],
  },
]);

const updateMetrics = () => {
  metrics.value = metrics.value.map((metric, index) => {
    const currentValue =
      index === 0
        ? props.data.current.cpu_percent
        : (props.data.current.ram_usage_bytes / props.data.current.ram_total_bytes) * 100;
    const pastValue =
      index === 0
        ? props.data.past.cpu_percent
        : (props.data.past.ram_usage_bytes / props.data.past.ram_total_bytes) * 100;

    const newValue = lerp(currentValue, pastValue);

    return {
      ...metric,
      value: `${newValue.toFixed(2)}%`,
      data: [...metric.data.slice(-10), newValue],
    };
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
