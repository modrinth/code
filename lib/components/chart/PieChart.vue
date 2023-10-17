<script setup>
import { ref } from 'vue'
import { Pie } from 'vue-chartjs'
import {
  Chart as ChartJS,
  Title,
  Tooltip,
  PieController,
  ArcElement,
  Legend,
  CategoryScale,
  LinearScale,
} from 'chart.js'

ChartJS.register(Title, Tooltip, PieController, ArcElement, Legend, CategoryScale, LinearScale)

const props = defineProps({
  data: {
    type: Object,
    required: true,
  },
})

const decimalToRgba = (decimalColor, alpha = 1) => {
  const red = (decimalColor >> 16) & 255
  const green = (decimalColor >> 8) & 255
  const blue = decimalColor & 255

  return `rgba(${red}, ${green}, ${blue}, ${alpha})`
}

const chartData = ref({
  labels: props.data.data.map((project) => project.title),
  datasets: [
    {
      label: props.data.title,
      backgroundColor: props.data.data.map((project) => decimalToRgba(project.color, 0.5)),
      borderColor: props.data.data.map((project) => decimalToRgba(project.color)),
      data: props.data.data.map((project) => project.data),
      fill: true,
    },
  ],
})

const chartOptions = ref({
  responsive: true,
  elements: {
    point: {
      radius: 0,
    },
  },
  plugins: {
    legend: {
      position: 'right',
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
  <Pie :options="chartOptions" :data="chartData" />
</template>

<style scoped lang="scss"></style>
