<script setup>
import { ref } from 'vue'
import { Bar } from 'vue-chartjs'
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  Legend,
  BarElement,
  CategoryScale,
  LinearScale,
} from 'chart.js'
import dayjs from 'dayjs'

ChartJS.register(Title, Tooltip, Legend, BarElement, CategoryScale, LinearScale)

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
    borderColor: decimalToRgba(project.color, 1),
    borderWidth: 2,
    borderSkipped: 'bottom',
    backgroundColor: decimalToRgba(project.color, 0.5),
    data: project.data,
  })),
})

const chartOptions = ref({
  responsive: true,
  scales: {
    x: {
      stacked: true,
      grid: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-button-bg'),
      },
      ticks: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
      },
    },
    y: {
      stacked: true,
      grid: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-button-bg'),
      },
      ticks: {
        color: getComputedStyle(document.documentElement).getPropertyValue('--color-base'),
      },
    },
  },
  interaction: {
    mode: 'index',
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
        size: 16,
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
      displayColors: false,
    },
  },
})
</script>

<template>
  <Bar id="my-chart-id" :options="chartOptions" :data="chartData" />
</template>

<style scoped lang="scss"></style>
