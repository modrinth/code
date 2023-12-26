<template>
  <div>
    <div v-if="analytics.error.value">
      {{ analytics.error.value }}
    </div>
    <div v-else class="graphs">
      <div class="graphs__vertical-bar">
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.downloads"
            ref="tinyDownloadChart"
            :title="`Downloads since ${dayjs(startDate).format('MMM D, YYYY')}`"
            color="var(--color-brand)"
            :value="formatNumber(analytics.formattedData.value.downloads.sum, false)"
            :data="analytics.formattedData.value.downloads.chart.sumData"
            :labels="analytics.formattedData.value.downloads.chart.labels"
            suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
            :class="`clickable button-base ${
              selectedChart === 'downloads' ? 'button-base__selected' : ''
            }`"
            :onclick="() => (selectedChart = 'downloads')"
            role="button"
          />
        </client-only>
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.views"
            ref="tinyViewChart"
            :title="`Page views since ${dayjs(startDate).format('MMM D, YYYY')}`"
            color="var(--color-blue)"
            :value="formatNumber(analytics.formattedData.value.views.sum, false)"
            :data="analytics.formattedData.value.views.chart.sumData"
            :labels="analytics.formattedData.value.views.chart.labels"
            suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z'/><circle cx='12' cy='12' r='3'/></svg>"
            :class="`clickable button-base ${
              selectedChart === 'views' ? 'button-base__selected' : ''
            }`"
            :onclick="() => (selectedChart = 'views')"
            role="button"
          />
        </client-only>
        <client-only>
          <CompactChart
            v-if="analytics.formattedData.value.revenue"
            ref="tinyRevenueChart"
            :title="`Revenue since ${dayjs(startDate).format('MMM D, YYYY')}`"
            color="var(--color-purple)"
            :value="formatMoney(analytics.formattedData.value.revenue.sum, false)"
            :data="analytics.formattedData.value.revenue.chart.sumData"
            :labels="analytics.formattedData.value.revenue.chart.labels"
            is-money
            :class="`clickable button-base ${
              selectedChart === 'revenue' ? 'button-base__selected' : ''
            }`"
            :onclick="() => (selectedChart = 'revenue')"
            role="button"
          />
        </client-only>
      </div>
      <div class="graphs__main-graph">
        <Card>
          <div class="graphs__main-graph-control">
            <DropdownSelect
              v-model="selectedRange"
              :options="selectableRanges"
              name="Time range"
              :display-name="(o: typeof selectableRanges[number] | undefined) => o?.label || 'Custom'"
            />
            <!-- <DropdownSelect
              v-model="selectedResolution"
              :options="selectableResoloutions"
              :display-name="(o: typeof selectableResoloutions[number] | undefined) => o?.label || 'Custom'"
            /> -->
          </div>
          <client-only>
            <Chart
              v-if="analytics.formattedData.value.downloads && selectedChart === 'downloads'"
              ref="downloadsChart"
              type="line"
              name="Download data"
              legend-position="right"
              :data="analytics.formattedData.value.downloads.chart.data"
              :labels="analytics.formattedData.value.downloads.chart.labels"
              suffix="<svg xmlns='http://www.w3.org/2000/svg' class='h-6 w-6' fill='none' viewBox='0 0 24 24' stroke='currentColor' stroke-width='2'><path stroke-linecap='round' stroke-linejoin='round' d='M4 16v1a3 3 0 003 3h10a3 3 0 003-3v-1m-4-4l-4 4m0 0l-4-4m4 4V4' /></svg>"
              :colors="analytics.formattedData.value.downloads.chart.colors"
            >
              <h2>Downloads</h2>
            </Chart>
            <Chart
              v-if="analytics.formattedData.value.views && selectedChart === 'views'"
              ref="viewsChart"
              type="line"
              name="View data"
              legend-position="right"
              :data="analytics.formattedData.value.views.chart.data"
              :labels="analytics.formattedData.value.views.chart.labels"
              suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><path d='M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z'/><circle cx='12' cy='12' r='3'/></svg>"
              :colors="analytics.formattedData.value.views.chart.colors"
            >
              <h2 class="">Views</h2>
            </Chart>
            <Chart
              v-if="analytics.formattedData.value.revenue && selectedChart === 'revenue'"
              ref="revenueChart"
              type="line"
              name="Revenue data"
              legend-position="right"
              :data="analytics.formattedData.value.revenue.chart.data"
              :labels="analytics.formattedData.value.revenue.chart.labels"
              is-money
              suffix="<svg xmlns='http://www.w3.org/2000/svg' width='24' height='24' viewBox='0 0 24 24' fill='none' stroke='currentColor' stroke-width='2' stroke-linecap='round' stroke-linejoin='round'><line x1='12' y1='2' x2='12' y2='22'></line><path d='M17 5H9.5a3.5 3.5 0 0 0 0 7h5a3.5 3.5 0 0 1 0 7H6'></path></svg>"
              :colors="analytics.formattedData.value.revenue.chart.colors"
            >
              <h2 class="">Revenue</h2>
            </Chart>
          </client-only>
        </Card>
        <div class="country-data">
          <Card
            v-if="
              analytics.formattedData.value?.downloadsByCountry && selectedChart === 'downloads'
            "
            class="country-downloads"
          >
            <label>
              <span class="label__title">Downloads by country</span>
            </label>
            <div class="country-values">
              <div
                v-for="[name, count] in analytics.formattedData.value.downloadsByCountry.data"
                :key="name"
                class="country-value"
              >
                <div class="country-flag-container">
                  <img
                    :src="
                      name.toLowerCase() === 'xx' || !name
                        ? 'https://cdn.modrinth.com/placeholder-banner.svg'
                        : countryCodeToFlag(name)
                    "
                    alt="Hidden country"
                    class="country-flag"
                  />
                </div>
                <div class="country-text">
                  <strong class="country-name"
                    ><template v-if="name.toLowerCase() === 'xx' || !name">Hidden</template>
                    <template v-else>{{ countryCodeToName(name) }}</template>
                  </strong>
                  <span class="data-point">{{ formatNumber(count) }}</span>
                </div>
                <div
                  v-tooltip="
                    formatPercent(count, analytics.formattedData.value.downloadsByCountry.sum)
                  "
                  class="percentage-bar"
                >
                  <span
                    :style="{
                      width: formatPercent(
                        count,
                        analytics.formattedData.value.downloadsByCountry.sum
                      ),
                      backgroundColor: 'var(--color-brand)',
                    }"
                  ></span>
                </div>
              </div>
            </div>
          </Card>
          <Card
            v-if="analytics.formattedData.value?.viewsByCountry && selectedChart === 'views'"
            class="country-downloads"
          >
            <label>
              <span class="label__title">Page views by country</span>
            </label>
            <div class="country-values">
              <div
                v-for="[name, count] in analytics.formattedData.value.viewsByCountry.data"
                :key="name"
                class="country-value"
              >
                <div class="country-flag-container">
                  <img
                    :src="`https://flagcdn.com/h240/${name.toLowerCase()}.png`"
                    :alt="name"
                    class="country-flag"
                  />
                </div>

                <div class="country-text">
                  <strong class="country-name">{{ countryCodeToName(name) }}</strong>
                  <span class="data-point">{{ formatNumber(count) }}</span>
                </div>
                <div
                  v-tooltip="
                    `${
                      Math.round(
                        (count / analytics.formattedData.value.viewsByCountry.sum) * 10000
                      ) / 100
                    }%`
                  "
                  class="percentage-bar"
                >
                  <span
                    :style="{
                      width: `${(count / analytics.formattedData.value.viewsByCountry.sum) * 100}%`,
                      backgroundColor: 'var(--color-blue)',
                    }"
                  ></span>
                </div>
              </div>
            </div>
          </Card>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { Card, formatMoney, formatNumber, DropdownSelect } from 'omorphia'
import dayjs from 'dayjs'
import { defineProps, ref, computed } from 'vue'
import { UiChartsCompactChart as CompactChart, UiChartsChart as Chart } from '#components'

const props = withDefaults(
  defineProps<{
    projects?: any[]
    /**
     * @deprecated Use `ranges` instead
     */
    resoloutions?: Record<string, number>
    ranges?: Record<number, [string, number] | string>
  }>(),
  {
    projects: undefined,
    resoloutions: () => defaultResoloutions,
    ranges: () => defaultRanges,
  }
)

const selectableRanges = Object.entries(props.ranges).map(([duration, extra]) => ({
  label: typeof extra === 'string' ? extra : extra[0],
  value: Number(duration),
  res: typeof extra === 'string' ? Number(duration) : extra[1],
}))

const selectedChart = ref('downloads')

// Chart refs
const downloadsChart = ref()
const viewsChart = ref()
const revenueChart = ref()
const tinyDownloadChart = ref()
const tinyViewChart = ref()
const tinyRevenueChart = ref()

const analytics = useFetchAllAnalytics(() => {
  downloadsChart.value?.resetChart()
  viewsChart.value?.resetChart()
  revenueChart.value?.resetChart()

  tinyDownloadChart.value?.resetChart()
  tinyViewChart.value?.resetChart()
  tinyRevenueChart.value?.resetChart()
}, props.projects)

const { startDate, endDate, timeRange, timeResolution } = analytics

const selectedRange = computed({
  get: () => {
    return (
      selectableRanges.find((option) => option.value === timeRange.value) || {
        label: 'Custom',
        value: timeRange.value,
      }
    )
  },
  set: (newRange: { label: string; value: number; res?: number }) => {
    timeRange.value = newRange.value
    startDate.value = Date.now() - timeRange.value * 60 * 1000
    endDate.value = Date.now()

    if (newRange?.res) {
      timeResolution.value = newRange.res
    }
  },
})
</script>

<script lang="ts">
const defaultResoloutions: Record<string, number> = {
  '5 minutes': 5,
  '30 minutes': 30,
  'An hour': 60,
  '12 hours': 720,
  'A day': 1440,
  'A week': 10080,
}

const defaultRanges: Record<number, [string, number] | string> = {
  30: ['Last 30 minutes', 1],
  60: ['Last hour', 5],
  720: ['Last 12 hours', 15],
  1440: ['Last day', 60],
  10080: ['Last week', 720],
  43200: ['Last month', 1440],
  129600: ['Last quarter', 10080],
  525600: ['Last year', 20160],
  1051200: ['Last two years', 40320],
}
</script>

<style scoped lang="scss">
.button-base {
  overflow: hidden;
}

.button-base__selected {
  color: var(--color-contrast);
  background-color: var(--color-brand-highlight);
  box-shadow: inset 0 0 0 transparent, 0 0 0 2px var(--color-brand);

  &:hover {
    background-color: var(--color-brand-highlight);
  }
}

.graphs {
  // Pages clip so we need to add a margin
  margin-left: 0.25rem;
  margin-top: 0.25rem;

  display: flex;
  flex-direction: column;

  .graphs__vertical-bar {
    flex-grow: 0;
    flex-shrink: 0;
    gap: 0.75rem;
    display: flex;
    margin-right: 0.1rem;
  }

  .graphs__main-graph {
    // Take up the rest of the width
    flex-grow: 1;

    display: grid;
    grid-template-columns: 1fr;

    .graphs__main-graph-control {
      display: flex;
      flex-direction: row;
      align-items: flex-end;
      justify-content: end;
      margin-bottom: var(--gap-md);
      gap: var(--gap-md);

      .animated-dropdown {
        width: auto;
      }
    }
  }
}

// Mobile
@media (max-width: 768px) {
  .graphs {
    flex-direction: column;
    gap: var(--gap-md);

    .graphs__vertical-bar {
      display: block;

      width: 100%;
      max-width: none;
    }

    .graphs__main-graph {
      display: block;
      overflow: hidden;
    }
  }
}

.country-flag-container {
  width: 40px;
  height: 27px;

  display: flex;
  justify-content: center;
  align-items: center;

  overflow: hidden;

  border: 1px solid var(--color-divider);
  border-radius: var(--radius-xs);
}

.country-flag {
  object-fit: cover;

  min-width: 100%;
  min-height: 100%;
}

.spark-data {
  display: grid;
  grid-template-columns: 1fr 1fr 1fr;
  gap: var(--gap-md);
}

.country-data {
  display: grid;
  grid-template-columns: 1fr;
  gap: var(--gap-md);
}

.country-values {
  display: flex;
  flex-direction: column;
  background-color: var(--color-bg);
  border-radius: var(--radius-sm);
  border: 1px solid var(--color-button-bg);
  gap: var(--gap-md);
  padding: var(--gap-md);
  margin-top: var(--gap-md);
  overflow-y: auto;
  max-height: 24rem;
}

.country-value {
  display: grid;
  grid-template-areas: 'flag text bar';
  grid-template-columns: auto 1fr 10rem;
  align-items: center;
  justify-content: space-between;
  width: 100%;
  gap: var(--gap-sm);

  .country-text {
    grid-area: text;
    display: flex;
    flex-direction: column;
    gap: var(--gap-xs);
  }
  .percentage-bar {
    grid-area: bar;
    width: 100%;
    height: 1rem;
    background-color: var(--color-raised-bg);
    border: 1px solid var(--color-button-bg);
    border-radius: var(--radius-sm);
    overflow: hidden;
    span {
      display: block;
      height: 100%;
    }
  }
}

@media (max-width: 768px) {
  .country-data {
    display: block;
  }

  .country-value {
    grid-template-columns: auto 1fr 5rem;
  }
}
</style>
