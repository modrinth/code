<script setup lang="ts">
import type {GameInstance} from '@/helpers/types'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import {DropdownIcon} from '@modrinth/assets'
import type {Version} from '@modrinth/utils'
import {computed, onMounted, ref} from 'vue'
import dayjs from 'dayjs'
import advancedFormat from 'dayjs/plugin/advancedFormat.js'
import type {Screenshot} from '@/helpers/screenshots.ts'
import {getAllProfileScreenshots} from '@/helpers/screenshots.ts'
import ScreenshotCard from '@/components/ui/ScreenshotCard.vue'

dayjs.extend(advancedFormat)

const props = defineProps<{
  instance: GameInstance
  options: InstanceType<typeof ContextMenu> | null
  offline: boolean
  playing: boolean
  versions: Version[]
  installed: boolean
}>()

const screenshots = ref<Screenshot[]>([])

onMounted(async () => {
  screenshots.value = (await getAllProfileScreenshots(props.instance.path)) ?? []
})

function groupAndSortByDate(items: Screenshot[]) {
  const today = dayjs().startOf('day')
  const yesterday = today.subtract(1, 'day')
  const map = new Map<string, { labelDate: dayjs.Dayjs; items: Screenshot[] }>()

  for (const shot of items) {
    const d = dayjs(shot.creation_date).startOf('day')
    let label: string
    if (d.isSame(today)) label = 'Today'
    else if (d.isSame(yesterday)) label = 'Yesterday'
    else label = dayjs(shot.creation_date).format('MMMM Do, YYYY')

    if (!map.has(label)) {
      map.set(label, {labelDate: d, items: []})
    }

    map.get(label)!.items.push(shot)
  }

  return Array.from(map.entries())
      .sort(([a, aData], [b, bData]) => {
        if (a === 'Today') return -1
        if (b === 'Today') return 1
        if (a === 'Yesterday') return -1
        if (b === 'Yesterday') return 1
        return bData.labelDate.unix() - aData.labelDate.unix()
      })
      .map(([label, {items}]) => [label, items] as const)
}

const screenshotsByDate = computed(() => groupAndSortByDate(screenshots.value))
const hasToday = computed(() => screenshotsByDate.value.some(([label]) => label === 'Today'))
</script>

<template>
  <div class="w-full p-5">
    <div
        v-if="!screenshots.length"
        class="flex flex-col items-center justify-center py-12 text-center"
    >
      <div class="text-lg font-medium mb-2">No screenshots yet</div>
      <div class="text-sm text-gray-500 dark:text-gray-400">
        Screenshots taken in-game will appear here
      </div>
    </div>

    <div v-else class="space-y-8">
      <template v-if="!hasToday">
        <details class="space-y-2" open>
          <summary class="cursor-pointer flex items-center justify-between">
            <h2
                class="text-xxl font-bold underline decoration-4 decoration-brand-green underline-offset-8"
            >
              Today
            </h2>
            <DropdownIcon
                class="w-5 h-5 transform transition-transform duration-200 group-open:rotate-180"
            />
          </summary>
          <p class="text-lg font-medium mb-2">You haven't taken any screenshots today.</p>
        </details>
      </template>

      <template v-for="[date, shots] in screenshotsByDate" :key="date">
        <details class="space-y-2" open>
          <summary class="cursor-pointer flex items-center justify-between">
            <h2
                class="text-xxl font-bold underline decoration-4 decoration-brand-green underline-offset-8"
            >
              {{ date }}
            </h2>
            <DropdownIcon
                class="w-5 h-5 transform transition-transform duration-200 group-open:rotate-180"
            />
          </summary>
          <div class="grid grid-cols-1 sm:grid-cols-2 lg:grid-cols-3 gap-4 pt-2">
            <ScreenshotCard v-for="s in shots" :key="s.path" :screenshot="s"/>
          </div>
        </details>
      </template>
    </div>
  </div>
</template>
