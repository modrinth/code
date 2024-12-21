<template>
  <div
    data-pyro-server-stats
    style="font-variant-numeric: tabular-nums"
    class="flex select-none flex-col items-center gap-6 md:flex-row"
  >
    <div
      v-for="(metric, index) in metrics"
      :key="index"
      class="relative isolate min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8"
    >
      <div
        class="relative z-10 -ml-3 w-fit rounded-xl px-3 py-1"
        :style="{
          backdropFilter: 'blur(6px)',
        }"
      >
        <div class="-mb-0.5 mt-0.5 flex flex-row items-center gap-2">
          <h2 class="m-0 -ml-0.5 text-3xl font-extrabold text-contrast">
            {{ metric.value }}
          </h2>
          <h3 class="relative z-10 text-sm font-normal text-secondary">/ {{ metric.max }}</h3>
        </div>
        <h3 class="relative z-10 flex items-center gap-2 text-base font-normal text-secondary">
          {{ metric.title }}
          <WarningIcon
            v-tooltip="getPotentialWarning(metric)"
            :style="{
              color: 'var(--color-orange)',
              width: '1.25rem',
              height: '1.25rem',
              display: getPotentialWarning(metric) ? 'block' : 'none',
            }"
          />
        </h3>
      </div>

      <component :is="metric.icon" class="absolute right-10 top-10 z-10" />
      <ClientOnly>
        <VueApexCharts
          v-if="
            metric.data.length && !(metric.title === 'Memory usage' && userPreferences.ramAsNumber)
          "
          ref="chart"
          type="area"
          height="142"
          :options="generateOptions(metric)"
          :series="[{ name: 'Chart', data: metric.data }]"
          class="chart chart-animation absolute bottom-0 left-0 right-0 w-full"
        />
      </ClientOnly>
    </div>

    <NuxtLink
      :to="`/servers/manage/${serverId}/files`"
      class="relative isolate min-h-[156px] w-full overflow-hidden rounded-2xl bg-bg-raised p-8 transition-transform duration-100 hover:scale-105 active:scale-100"
    >
      <div class="flex flex-row items-center gap-2">
        <h2 class="m-0 -ml-0.5 mt-1 text-3xl font-extrabold text-contrast">
          {{ formatBytes(animatedStorageUsage) }}
        </h2>
        <!-- <h3 class="relative z-10 text-sm font-normal text-secondary">
          / {{ formatBytes(props.data.current.storage_total_bytes) }}
        </h3> -->
      </div>
      <h3 class="relative z-10 text-base font-normal text-secondary">Storage usage</h3>

      <FolderOpenIcon class="absolute right-10 top-10 size-8" />
    </NuxtLink>
  </div>
</template>

<script setup lang="ts">
import { ref, onMounted, watch } from "vue";
import { FolderOpenIcon, CPUIcon, DBIcon } from "@modrinth/assets";
import { useStorage } from "@vueuse/core";
import type { Stats } from "~/types/servers";
import WarningIcon from "~/assets/images/utils/issues.svg?component";

const route = useNativeRoute();
const serverId = route.params.id;

const userPreferences = useStorage(`pyro-server-${serverId}-preferences`, {
  ramAsNumber: false,
  autoRestart: false,
  backupWhileRunning: false,
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

const animatedStorageUsage = ref(0);

const animateValue = (start: number, end: number, duration: number): void => {
  let startTimestamp: number | null = null;
  const step = (timestamp: number) => {
    if (!startTimestamp) startTimestamp = timestamp;
    const progress = Math.min((timestamp - startTimestamp) / duration, 1);
    animatedStorageUsage.value = Math.floor(progress * (end - start) + start);
    if (progress < 1) {
      requestAnimationFrame(step);
    }
  };
  requestAnimationFrame(step);
};

onMounted(() => {
  animateValue(0, props.data.current.storage_usage_bytes, 250);
});

watch(
  () => props.data.current.storage_usage_bytes,
  (newValue, oldValue) => {
    animateValue(oldValue, newValue, 250);
  },
);

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
    max: userPreferences.value.ramAsNumber
      ? formatBytes(props.data.current.ram_total_bytes)
      : "100%",
    icon: markRaw(DBIcon),
    data: [] as number[],
  },
]);

const updateMetrics = () => {
  console.log(props.data.current.ram_usage_bytes);
  metrics.value = metrics.value.map((metric, index) => {
    if (userPreferences.value.ramAsNumber && index === 1) {
      return {
        ...metric,
        value: formatBytes(props.data.current.ram_usage_bytes),
        data: [...metric.data.slice(-10), props.data.current.ram_usage_bytes],
        max: formatBytes(props.data.current.ram_total_bytes),
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
        // data: [36, 36],
      };
    }
  });
};

// aww, you gotta give em that rinth tuah, mod on that thang
const getPotentialWarning = (metric: (typeof metrics.value)[0]) => {
  // make all words in the string lowercase, unless the word is in all caps
  const split = metric.title.split(" ");
  const title = split
    .map((word) => {
      if (word === word.toUpperCase()) {
        return word;
      }
      return word.toLowerCase();
    })
    .join(" ");
  let data = metric.data.at(-1) || 0;
  if (userPreferences.value.ramAsNumber) {
    data = (props.data.current.ram_usage_bytes / props.data.current.ram_total_bytes) * 100;
  }
  switch (true) {
    case data >= 90:
      return `Your server's ${title} is very high.`;
    default:
      return "";
  }
};

const generateOptions = (metric: (typeof metrics.value)[0]) => {
  let color = "var(--color-brand)";
  let data = metric.data.at(-1) || 0;
  if (userPreferences.value.ramAsNumber) {
    data = (props.data.current.ram_usage_bytes / props.data.current.ram_total_bytes) * 100;
  }
  switch (true) {
    case data >= 90:
      color = "var(--color-red)";
      break;
    case data >= 80:
      color = "var(--color-orange)";
      break;
  }
  return {
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
      colors: [color],
      type: "gradient",
      opacity: 1,
      gradient: {
        shade: "light",
        type: "vertical",
        shadeIntensity: 0,
        gradientToColors: [color],
        inverseColors: true,
        opacityFrom: 0.5,
        opacityTo: 0,
        stops: [0, 100],
        colorStops: [],
      },
    },
    grid: { show: false },
    legend: { show: false },
    colors: [color],
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
  };
};

// watch(
//   metrics,
//   () => {
//     console.log(metrics.value[0].data.at(-1));
//   },
//   {
//     deep: true,
//     immediate: true,
//   },
// );

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
