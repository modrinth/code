<template>
  <div data-pyro-server-stats class="flex flex-row items-center gap-6">
    <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised">
      <div class="p-8">
        <div class="flex flex-row items-center gap-2">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
            {{
              lerp(
                Math.round(data.current.cpu_percent * 100) / 100,
                Math.round(data.past.cpu_percent * 100) / 100,
              ).toFixed(2)
            }}%
          </h2>
          <ChevronRightIcon />
        </div>
        <h3>CPU usage</h3>
        <CPUIcon />
      </div>
      <ClientOnly>
        <VueApexCharts
          ref="chart"
          type="area"
          height="150"
          :options="chartOptions"
          :series="[
            {
              name: 'Chart',
              data: data.graph.cpu,
            },
          ]"
          class="chart absolute bottom-0 w-full"
        />
      </ClientOnly>
    </div>

    <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised">
      <div class="p-8">
        <div class="flex flex-row items-center gap-2">
          <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
            {{
              lerp(
                Math.floor((data.current.ram_usage_bytes / data.current.ram_total_bytes) * 100),
                Math.floor((data.past.ram_usage_bytes / data.past.ram_total_bytes) * 100),
              )
            }}%
          </h2>
          <ChevronRightIcon />
        </div>
        <h3>Memory usage</h3>

        <DBIcon />
      </div>
      <ClientOnly>
        <VueApexCharts
          ref="chart"
          type="area"
          height="150"
          :options="chartOptions"
          :series="[
            {
              name: 'Chart',
              data: data.graph.ram,
            },
          ]"
          class="chart absolute bottom-0 w-full"
        />
      </ClientOnly>
    </div>

    <div class="relative min-h-[230px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8">
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 text-3xl font-extrabold text-[var(--color-contrast)]">
          {{ (Math.round((data.current.storage_total_bytes / 1e9) * 100) / 100).toFixed(2) }} GB
        </h2>
        <!-- make mb when not decimal -->
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
import type { Stats } from "~/types/servers";

import { FileTextIcon, FolderOpenIcon, ChevronRightIcon, CPUIcon, DBIcon } from "@modrinth/assets";

const VueApexCharts = defineAsyncComponent(() => import("vue3-apexcharts"));

const route = useNativeRoute();
const serverId = route.params.id;

defineProps({
  data: {
    type: Object as PropType<Stats>,
    required: true,
  },
});

const lerp = (a: number, b: number) => {
  // fix this :)
  return a;
};

const chartOptions = ref({
  chart: {
    id: "stats",
    fontFamily:
      "Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Oxygen, Ubuntu, Roboto, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif",
    foreColor: "var(--color-base)",
    toolbar: {
      show: false,
    },
    zoom: {
      enabled: false,
      autoScaleYaxis: false,
    },
    sparkline: {
      enabled: true,
    },
    animations: {
      enabled: true,
      easing: "easeinout",
      dynamicAnimation: {
        speed: 1000,
      },
    },
  },
  stroke: {
    curve: "smooth",
  },
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
  grid: {
    show: false,
  },
  legend: {
    show: false,
  },
  colors: ["var(--color-brand)"],
  dataLabels: {
    enabled: false,
  },
  xaxis: {
    categories: ["1", "2", "3", "4", "5", "6", "7 ", "8", "9", "10"],
    labels: {
      show: false,
    },
    axisTicks: {
      show: false,
    },
    tooltip: {
      enabled: false,
    },
  },
  yaxis: {
    labels: {
      show: false,
    },
    axisBorder: {
      show: false,
    },
    axisTicks: {
      show: false,
    },
    tooltip: {
      enabled: false,
    },
    tickAmount: 10,
    min: 0,
    max: 100,
    stepSize: 5,
  },
  tooltip: {
    enabled: false,
  },
});
</script>
