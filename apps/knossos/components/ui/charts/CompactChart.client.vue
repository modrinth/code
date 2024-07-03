<script setup>
import { Card } from 'omorphia'
import VueApexCharts from 'vue3-apexcharts'

// let VueApexCharts
// if (process.client) {
//   VueApexCharts = defineAsyncComponent(() => import('vue3-apexcharts'))
// }

const props = defineProps({
  value: {
    type: String,
    default: '',
  },
  title: {
    type: String,
    default: '',
  },
  data: {
    type: Array,
    default: () => [],
  },
  labels: {
    type: Array,
    default: () => [],
  },
  prefix: {
    type: String,
    default: '',
  },
  suffix: {
    type: String,
    default: '',
  },
  isMoney: {
    type: Boolean,
    default: false,
  },
  color: {
    type: String,
    default: 'var(--color-brand)',
  },
})

// no grid lines, no toolbar, no legend, no data labels
const chartOptions = {
  chart: {
    id: props.title,
    fontFamily:
      'Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Oxygen, Ubuntu, Roboto, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif',
    foreColor: 'var(--color-base)',
    toolbar: {
      show: false,
    },
    zoom: {
      enabled: false,
    },
    sparkline: {
      enabled: true,
    },
    parentHeightOffset: 0,
  },
  stroke: {
    curve: 'smooth',
    width: 2,
  },
  fill: {
    colors: [props.color],
    type: 'gradient',
    opacity: 1,
    gradient: {
      shade: 'light',
      type: 'vertical',
      shadeIntensity: 0,
      gradientToColors: [props.color],
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
  colors: [props.color],
  dataLabels: {
    enabled: false,
  },
  xaxis: {
    type: 'datetime',
    categories: props.labels,
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
  },
  tooltip: {
    enabled: false,
  },
}

const chart = ref(null)

const resetChart = () => {
  chart.value?.updateSeries([...props.data])
  chart.value?.updateOptions({
    xaxis: {
      categories: props.labels,
    },
  })
  chart.value?.resetSeries()
}

defineExpose({
  resetChart,
})
</script>

<template>
  <Card class="compact-chart">
    <h1 class="value">
      {{ value }}
    </h1>
    <div class="subtitle">
      {{ title }}
    </div>
    <div class="chart">
      <VueApexCharts ref="chart" type="area" :options="chartOptions" :series="data" height="70" />
    </div>
  </Card>
</template>

<style scoped lang="scss">
.compact-chart {
  display: flex;
  flex-direction: column;

  gap: var(--gap-xs);
  border: 1px solid var(--color-button-bg);
  border-radius: var(--radius-md);
  background-color: var(--color-raised-bg);
  box-shadow: var(--shadow-floating);

  color: var(--color-base);
  font-size: var(--font-size-nm);

  width: 100%;

  padding-top: var(--gap-xl);
  padding-bottom: 0;

  .value {
    margin: 0;
  }
}

.chart {
  // width: calc(100% + 3rem);
  margin: 0 -1.5rem 0.25rem -1.5rem;
}

svg {
  width: 100%;
  height: 100%;
}

:deep(.apexcharts-menu),
:deep(.apexcharts-tooltip),
:deep(.apexcharts-yaxistooltip) {
  background: var(--color-raised-bg) !important;
  border-radius: var(--radius-sm) !important;
  border: 1px solid var(--color-button-bg) !important;
  box-shadow: var(--shadow-floating) !important;
  font-size: var(--font-size-nm) !important;
}

:deep(.apexcharts-graphical) {
  width: 100%;
}

:deep(.apexcharts-tooltip) {
  .bar-tooltip {
    display: flex;
    flex-direction: column;
    gap: var(--gap-xs);
    padding: var(--gap-sm);

    .card-divider {
      margin: var(--gap-xs) 0;
    }

    .label {
      display: flex;
      flex-direction: row;
      align-items: center;
    }

    .value {
      display: flex;
      flex-direction: row;
      align-items: center;
      gap: var(--gap-xs);
      color: var(--color-base);
    }

    .list-entry {
      display: flex;
      flex-direction: row;
      align-items: center;
      justify-content: space-between;
      gap: var(--gap-md);
    }

    .circle {
      width: 0.5rem;
      height: 0.5rem;
      border-radius: 50%;
      display: inline-block;
      margin-right: var(--gap-sm);
    }

    svg {
      height: 1em;
      width: 1em;
    }

    .divider {
      font-size: var(--font-size-lg);
      font-weight: 400;
    }
  }
}

.legend {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-md);
  justify-content: center;
}

:deep(.apexcharts-grid-borders) {
  line {
    stroke: var(--color-button-bg) !important;
  }
}

:deep(.apexcharts-xaxis) {
  line {
    stroke: none;
  }
}

.legend-checkbox :deep(.checkbox.checked) {
  background-color: var(--color);
}
</style>
