<!-- eslint-disable no-console -->
<script setup>
import dayjs from 'dayjs'
import { formatNumber } from '@modrinth/utils'
import { defineAsyncComponent, ref } from 'vue'
import Button from '../base/Button.vue'
import Checkbox from '../base/Checkbox.vue'

const VueApexCharts = defineAsyncComponent(() => import('vue3-apexcharts'))

const props = defineProps({
  name: {
    type: String,
    required: true,
  },
  labels: {
    type: Array,
    required: true,
  },
  data: {
    type: Array,
    required: true,
  },
  formatLabels: {
    type: Function,
    default: (label) => dayjs(label).format('MMM D'),
  },
  colors: {
    type: Array,
    default: () => [
      'var(--color-brand)',
      'var(--color-blue)',
      'var(--color-purple)',
      'var(--color-red)',
      'var(--color-orange)',
    ],
  },
  prefix: {
    type: String,
    default: '',
  },
  suffix: {
    type: String,
    default: '',
  },
  hideToolbar: {
    type: Boolean,
    default: false,
  },
  hideLegend: {
    type: Boolean,
    default: false,
  },
  stacked: {
    type: Boolean,
    default: false,
  },
  type: {
    type: String,
    default: 'bar',
  },
  hideTotal: {
    type: Boolean,
    default: false,
  },
})

const chartOptions = ref({
  chart: {
    id: props.name,
    fontFamily:
      'Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Oxygen, Ubuntu, Roboto, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif',
    foreColor: 'var(--color-base)',
    selection: {
      enabled: true,
      fill: {
        color: 'var(--color-brand)',
      },
    },
    toolbar: {
      show: false,
    },
    stacked: props.stacked,
  },
  xaxis: {
    type: 'datetime',
    categories: props.labels,
    labels: {
      style: {
        borderRadius: 'var(--radius-sm)',
      },
    },
    axisTicks: {
      show: false,
    },
    tooltip: {
      enabled: false,
    },
  },
  yaxis: {
    tooltip: {
      enabled: false,
    },
  },
  colors: props.colors,
  dataLabels: {
    enabled: false,
    background: {
      enabled: true,
      borderRadius: 20,
    },
  },
  grid: {
    borderColor: 'var(--color-button-bg)',
    tickColor: 'var(--color-button-bg)',
  },
  legend: {
    show: false,
  },
  markers: {
    size: 0,
    strokeColor: 'var(--color-contrast)',
    strokeWidth: 3,
    strokeOpacity: 1,
    fillOpacity: 1,
    hover: {
      size: 6,
    },
  },
  plotOptions: {
    bar: {
      columnWidth: '80%',
      endingShape: 'rounded',
      borderRadius: 5,
      borderRadiusApplication: 'end',
      borderRadiusWhenStacked: 'last',
    },
  },
  tooltip: {
    custom({ series, seriesIndex, dataPointIndex, w }) {
      console.log(seriesIndex, w)
      return (
        `<div class="bar-tooltip">` +
        `<div class="seperated-entry title">` +
        `<div class="label">${props.formatLabels(
          w.globals.lastXAxis.categories[dataPointIndex],
        )}</div>${
          !props.hideTotal
            ? `<div class="value">
        ${props.prefix}
        ${formatNumber(series.reduce((a, b) => a + b[dataPointIndex], 0).toString(), false)}
        ${props.suffix}
        </div>`
            : ``
        }</div><hr class="card-divider" />${series
          .map((value, index) =>
            value[dataPointIndex] > 0
              ? `<div class="list-entry">
                <span class="circle" style="background-color: ${w.globals.colors[index]}"> </span>
                <div class="label">
                  ${w.globals.seriesNames[index]}
                </div>
                <div class="value">
                  ${props.prefix}
                  ${formatNumber(value[dataPointIndex], false)}
                  ${props.suffix}
                </div>
              </div>`
              : '',
          )
          .reverse()
          .reduce((a, b) => a + b)}</div>`
      )
    },
  },
})

const chart = ref(null)

const legendValues = ref(
  [...props.data].map((project, index) => {
    return { name: project.name, visible: true, color: props.colors[index] }
  }),
)

const flipLegend = (legend, newVal) => {
  legend.visible = newVal
  chart.value.toggleSeries(legend.name)
}

const downloadCSV = () => {
  const csvContent = `data:text/csv;charset=utf-8,${props.labels.join(',')}\n${props.data
    .map((project) => project.data.join(','))
    .reduce((a, b) => `${a}\n${b}`)}`

  const encodedUri = encodeURI(csvContent)
  const link = document.createElement('a')
  link.setAttribute('href', encodedUri)
  link.setAttribute('download', `${props.name}.csv`)
  document.body.appendChild(link) // Required for FF

  link.click()
}

const resetChart = () => {
  chart.value.resetSeries()
  legendValues.value.forEach((legend) => {
    legend.visible = true
  })
}

defineExpose({
  resetChart,
  downloadCSV,
  flipLegend,
})
</script>

<template>
  <div class="bar-chart">
    <div class="title-bar">
      <slot />
      <div v-if="!hideToolbar" class="toolbar">
        <Button v-tooltip="'Download data as CSV'" icon-only @click="downloadCSV">
          <!-- <DownloadIcon /> -->
        </Button>
        <Button v-tooltip="'Reset chart'" icon-only @click="resetChart">
          <!-- <UpdatedIcon /> -->
        </Button>
        <slot name="toolbar" />
      </div>
    </div>
    <VueApexCharts ref="chart" :type="type" :options="chartOptions" :series="data" class="chart" />
    <div v-if="!hideLegend" class="legend">
      <Checkbox
        v-for="legend in legendValues"
        :key="legend.name"
        class="legend-checkbox"
        :style="`--color: ${legend.color};`"
        :model-value="legend.visible"
        @update:model-value="(newVal) => flipLegend(legend, newVal)"
      >
        {{ legend.name }}
      </Checkbox>
    </div>
  </div>
</template>

<style scoped lang="scss">
.chart {
  width: 100%;
  height: 100%;
}

svg {
  width: 100%;
  height: 100%;
}

.bar-chart {
  width: 100%;
  height: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;
}

.title-bar {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-xs);
}

.toolbar {
  display: flex;
  flex-direction: row;
  gap: var(--gap-xs);
  z-index: 1;
  margin-left: auto;
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

:deep(.apexcharts-grid-borders) {
  line {
    stroke: var(--color-button-bg) !important;
  }
}

:deep(.apexcharts-yaxistooltip),
:deep(.apexcharts-xaxistooltip) {
  background: var(--color-raised-bg) !important;
  border-radius: var(--radius-sm) !important;
  border: 1px solid var(--color-button-bg) !important;
  font-size: var(--font-size-nm) !important;
  color: var(--color-base) !important;

  .apexcharts-xaxistooltip-text {
    font-size: var(--font-size-nm) !important;
    color: var(--color-base) !important;
  }
}

:deep(.apexcharts-yaxistooltip-left:after) {
  border-left-color: var(--color-raised-bg) !important;
}

:deep(.apexcharts-yaxistooltip-left:before) {
  border-left-color: var(--color-button-bg) !important;
}

:deep(.apexcharts-xaxistooltip-bottom:after) {
  border-bottom-color: var(--color-raised-bg) !important;
}

:deep(.apexcharts-xaxistooltip-bottom:before) {
  border-bottom-color: var(--color-button-bg) !important;
}

:deep(.apexcharts-menu-item) {
  border-radius: var(--radius-sm) !important;
  padding: var(--gap-xs) var(--gap-sm) !important;

  &:hover {
    transition: all 0.3s !important;
    color: var(--color-accent-contrast) !important;
    background: var(--color-brand) !important;
  }
}

:deep(.apexcharts-tooltip) {
  .bar-tooltip {
    min-width: 10rem;
    display: flex;
    flex-direction: column;
    gap: var(--gap-xs);
    padding: var(--gap-sm);

    .card-divider {
      margin: var(--gap-xs) 0;
    }

    .seperated-entry {
      display: flex;
      flex-direction: row;
      justify-content: space-between;
      align-items: center;
    }

    .title {
      font-weight: bolder;
    }

    .label {
      color: var(--color-contrast);
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
      font-size: var(--font-size-sm);

      .value {
        margin-left: auto;
      }

      .label {
        margin-right: var(--gap-xl);
      }
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
  }
}

.legend {
  display: flex;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-md);
  justify-content: center;
}

.legend-checkbox :deep(.checkbox.checked) {
  background-color: var(--color);
}
</style>
