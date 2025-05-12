<script setup lang="ts">
import type { GameInstance } from '@/helpers/types'
import type ContextMenu from '@/components/ui/ContextMenu.vue'
import { DropdownIcon } from '@modrinth/assets'
import type { Version } from '@modrinth/utils'
import { computed, ref } from 'vue'
import dayjs from 'dayjs'
import advancedFormat from 'dayjs/plugin/advancedFormat.js'
import type { Screenshot } from '@/helpers/screenshots.ts'
import {
  getScreenshotData,
  getScreenshotFileName,
  getAllProfileScreenshots,
  openProfileScreenshot,
} from '@/helpers/screenshots.ts'
import ScreenshotCard from '@/components/ui/ScreenshotCard.vue'
import type {
  GalleryEntry,
  NavigationFunction,
  OpenExternallyFunction,
} from '@modrinth/ui/src/components/modal/ImagePreviewModal.vue'
import ImagePreviewModal from '@modrinth/ui/src/components/modal/ImagePreviewModal.vue'
import { useNotifications } from '@/store/state'

dayjs.extend(advancedFormat)

const props = defineProps<{
  instance: GameInstance
  options: InstanceType<typeof ContextMenu> | null
  offline: boolean
  playing: boolean
  versions: Version[]
  installed: boolean
}>()

const notifications = useNotifications()
const screenshots = ref<Screenshot[]>((await getAllProfileScreenshots(props.instance.path)) ?? [])
const imagePreviewModal = ref<typeof ImagePreviewModal>()

function groupAndSortByDate(items: Screenshot[]): readonly [string, Screenshot[]][] {
  const todayTS = dayjs().startOf('day').valueOf()
  const yesterdayTS = dayjs().subtract(1, 'day').startOf('day').valueOf()

  const groups = new Map<number, Screenshot[]>()
  for (const shot of items) {
    const ts = dayjs(shot.creation_date).startOf('day').valueOf()
    const bucket = groups.get(ts)
    if (bucket) bucket.push(shot)
    else groups.set(ts, [shot])
  }

  const sortedTS = Array.from(groups.keys()).sort((a, b) => b - a)
  return sortedTS.map((ts) => {
    let label: string
    if (ts === todayTS) label = 'Today'
    else if (ts === yesterdayTS) label = 'Yesterday'
    else label = dayjs(ts).format('MMMM Do, YYYY')

    return [label, groups.get(ts)!] as const
  })
}

function markDeleted(s: Screenshot): void {
  screenshots.value = screenshots.value.filter((shot) => shot.path !== s.path)
}

async function navigateScreenshot(screenshot: Screenshot, offset: number): Promise<GalleryEntry> {
  const list = screenshots.value
  const idx = list.findIndex((s) => s.path === screenshot.path)
  const newIdx = (idx + offset + list.length) % list.length
  const next = list[newIdx]

  return {
    src: `data:image/png;base64,${await getScreenshotData(props.instance.path, next)}`,
    alt: getScreenshotFileName(next.path),
    key: {
      ...next,
      title: getScreenshotFileName(next.path),
      description: `Taken on ${dayjs(next.creation_date).format('MMMM Do, YYYY')}`,
    },
  }
}

const viewNextScreenshot: NavigationFunction = ((s) =>
  navigateScreenshot(s, 1)) as NavigationFunction

const viewPreviousScreenshot: NavigationFunction = ((s) =>
  navigateScreenshot(s, -1)) as NavigationFunction

const openExternally: OpenExternallyFunction = (async (src: string, screenshot: Screenshot) => {
  const result = await openProfileScreenshot(props.instance.path, screenshot)
  if (!result) {
    notifications.addNotification({
      title: 'Unable to open screenshot in folder.',
      type: 'error',
    })
  }
}) as OpenExternallyFunction

const screenshotsByDate = computed(() => groupAndSortByDate(screenshots.value))
const hasToday = computed(() => screenshotsByDate.value.some(([label]) => label === 'Today'))
</script>

<template>
  <div>
    <ImagePreviewModal
      ref="imagePreviewModal"
      :next="viewNextScreenshot"
      :prev="viewPreviousScreenshot"
      :open-externally="openExternally"
      :open-externally-tooltip="'Open in containing folder'"
    />
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
          <details class="group space-y-2" open>
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
          <details class="group space-y-2" open>
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
              <ScreenshotCard
                v-for="s in shots"
                :key="s.path"
                :screenshot="s"
                :profile-path="instance.path"
                :image-preview-modal="imagePreviewModal!"
                @deleted="markDeleted(s)"
              />
            </div>
          </details>
        </template>
      </div>
    </div>
  </div>
</template>
