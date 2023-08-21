<template>
  <Line :options="chartOptions" :data="chartData" />
</template>

<script setup>
import { ref } from 'vue'
import { Line } from 'vue-chartjs'
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  PointElement,
  LineElement,
  CategoryScale,
  LinearScale,
  Filler,
} from 'chart.js'
import dayjs from 'dayjs'

ChartJS.register(Title, Tooltip, PointElement, LineElement, CategoryScale, LinearScale, Filler)

const props = defineProps({
  data: {
    type: Object,
    required: true,
  },
  formatLabels: {
    type: Function,
    default: (label) => dayjs(label).format('MMM D'),
  },
})

const decimalToRgba = (decimalColor, alpha = 0.75) => {
  const red = (decimalColor >> 16) & 255
  const green = (decimalColor >> 8) & 255
  const blue = decimalColor & 255

  return `rgba(${red}, ${green}, ${blue}, ${alpha})`
}

const chartData = ref({
  labels: props.data.labels.map((date) => props.formatLabels(date)),
  datasets: props.data.data.map((project) => ({
    label: project.title,
    backgroundColor: decimalToRgba(project.color, 0.75),
    borderColor: decimalToRgba(project.color),
    data: project.data,
  })),
})

const chartOptions = ref({
  responsive: true,
  scales: {
    x: {
      grid: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-button-bg'),
      },
      ticks: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
      },
    },
    y: {
      grid: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-button-bg'),
      },
      ticks: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
      },
    },
  },
  interaction: {
    mode: 'x',
  },
  plugins: {
    legend: {
      position: 'right',
      align: 'start',
      labels: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
        font: {
          size: 12,
          family: 'Inter',
        },
      },
    },
    tooltip: {
      position: 'nearest',
      backgroundColor: getComputedStyle(document.documentElement).getPropertyValue(
        '--color-raised-bg'
      ),
      borderColor: getComputedStyle(document.documentElement).getPropertyValue('--color-button-bg'),
      borderWidth: 1,
      titleColor: getComputedStyle(document.documentElement).getPropertyValue('--color-contrast'),
      titleFont: {
        size: 14,
        family: 'Inter',
      },
      bodyColor: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
      bodyFont: {
        size: 12,
        family: 'Inter',
      },
      boxPadding: 8,
      intersect: false,
      padding: 12,
    },
  },
})

/*
The data for the graph should look like this

downloads, views, likes = {
  dates: [ '2021-01-01', '2021-01-02', '2021-01-03' ], // Last 2 weeks
  data: [
    {
      title: projectName,
      color: projectColor,
      data: [ ... ],
    },
    ...
  ]
}
 */
</script>
