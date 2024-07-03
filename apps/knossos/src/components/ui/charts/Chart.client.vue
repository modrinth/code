<script setup>
import dayjs from 'dayjs'
import { formatNumber, formatMoney } from 'omorphia'
import VueApexCharts from 'vue3-apexcharts'

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
  isMoney: {
    type: Boolean,
    default: false,
  },
  legendPosition: {
    type: String,
    default: 'right',
  },
  xAxisType: {
    type: String,
    default: 'datetime',
  },
  percentStacked: {
    type: Boolean,
    default: false,
  },
  horizontalBar: {
    type: Boolean,
    default: false,
  },
  disableAnimations: {
    type: Boolean,
    default: false,
  },
})

function formatTooltipValue(value, props) {
  return props.isMoney ? formatMoney(value, false) : formatNumber(value, false)
}

function generateListEntry(value, index, _, w, props) {
  const color = w.globals.colors?.[index]

  return `<div class="list-entry">
    <span class="circle" style="background-color: ${color}"></span>
    <div class="label">
      ${w.globals.seriesNames[index]}
    </div>
    <div class="value">
      ${props.prefix}${formatTooltipValue(value, props)}${props.suffix}
    </div>
  </div>`
}

function generateTooltip({ series, seriesIndex, dataPointIndex, w }, props) {
  const label = w.globals.lastXAxis.categories?.[dataPointIndex]

  const formattedLabel = props.formatLabels(label)

  let tooltip = `<div class="bar-tooltip">
    <div class="seperated-entry title">
      <div class="label">${formattedLabel}</div>`

  // Logic for total and percent stacked
  if (!props.hideTotal) {
    if (props.percentStacked) {
      const total = series.reduce((a, b) => a + (b?.[dataPointIndex] || 0), 0)
      const percentValue = (100 * series[seriesIndex][dataPointIndex]) / total
      tooltip += `<div class="value">${props.prefix}${formatNumber(percentValue)}%${
        props.suffix
      }</div>`
    } else {
      const totalValue = series.reduce((a, b) => a + (b?.[dataPointIndex] || 0), 0)
      tooltip += `<div class="value">${props.prefix}${formatTooltipValue(totalValue, props)}${
        props.suffix
      }</div>`
    }
  }

  tooltip += '</div><hr class="card-divider" />'

  // Logic for generating list entries
  if (props.percentStacked) {
    tooltip += generateListEntry(
      series[seriesIndex][dataPointIndex],
      seriesIndex,
      seriesIndex,
      w,
      props
    )
  } else {
    const returnTopN = 5

    const listEntries = series
      .map((value, index) => [
        value[dataPointIndex],
        generateListEntry(value[dataPointIndex], index, seriesIndex, w, props),
      ])
      .filter((value) => value[0] > 0)
      .sort((a, b) => b[0] - a[0])
      .slice(0, returnTopN) // Return only the top X entries
      .map((value) => value[1])
      .join('')

    tooltip += listEntries
  }

  tooltip += '</div>'
  return tooltip
}

const chartOptions = computed(() => {
  return {
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
      stackType: props.percentStacked ? '100%' : 'normal',
      zoom: {
        autoScaleYaxis: true,
      },
      animations: {
        enabled: props.disableAnimations,
      },
    },
    xaxis: {
      type: props.xAxisType,
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
      show: !props.hideLegend,
      position: props.legendPosition,
      showForZeroSeries: false,
      showForSingleSeries: false,
      showForNullSeries: false,
      fontSize: 'var(--font-size-nm)',
      fontFamily:
        'Inter, -apple-system, BlinkMacSystemFont, Segoe UI, Oxygen, Ubuntu, Roboto, Cantarell, Fira Sans, Droid Sans, Helvetica Neue, sans-serif',
      onItemClick: {
        toggleDataSeries: true,
      },
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
        horizontal: props.horizontalBar,
        columnWidth: '80%',
        endingShape: 'rounded',
        borderRadius: 5,
        borderRadiusApplication: 'end',
        borderRadiusWhenStacked: 'last',
      },
    },
    stroke: {
      curve: 'smooth',
      width: 2,
    },
    tooltip: {
      custom: (d) => generateTooltip(d, props),
    },
    fill:
      props.type === 'area'
        ? {
            colors: props.colors,
            type: 'gradient',
            opacity: 1,
            gradient: {
              shade: 'light',
              type: 'vertical',
              shadeIntensity: 0,
              gradientToColors: props.colors,
              inverseColors: true,
              opacityFrom: 0.5,
              opacityTo: 0,
              stops: [0, 100],
              colorStops: [],
            },
          }
        : {},
  }
})

const chart = ref(null)

const legendValues = ref(
  [...props.data].map((project, index) => {
    return { name: project.name, visible: true, color: props.colors[index] }
  })
)

const flipLegend = (legend, newVal) => {
  legend.visible = newVal
  chart.value.toggleSeries(legend.name)
}

const resetChart = () => {
  if (!chart.value) return
  chart.value.updateSeries([...props.data])
  chart.value.updateOptions({
    xaxis: {
      categories: props.labels,
    },
  })
  chart.value.resetSeries()
  legendValues.value.forEach((legend) => {
    legend.visible = true
  })
}

defineExpose({
  resetChart,
  flipLegend,
})
</script>

<template>
  <VueApexCharts ref="chart" :type="type" :options="chartOptions" :series="data" class="chart" />
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

.btn {
  svg {
    width: 1.25rem;
    height: 1.25rem;
  }
}

.bar-chart {
  width: 100%;
  display: flex;
  flex-direction: column;
  justify-content: center;

  overflow: hidden;
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
      margin-right: var(--gap-xl);
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
    }

    .circle {
      width: 0.75rem;
      height: 0.75rem;
      border-radius: 50%;
      display: inline-block;
      margin-right: var(--gap-sm);
      border: 2px solid var(--color-base);
    }

    svg {
      height: 1em;
      width: 1em;
    }
  }
}

.legend {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  align-items: center;
  gap: var(--gap-lg);
  justify-content: center;
}

:deep(.checkbox) {
  white-space: nowrap;
}

.legend-checkbox :deep(.checkbox.checked) {
  background-color: var(--color);
}
</style>
