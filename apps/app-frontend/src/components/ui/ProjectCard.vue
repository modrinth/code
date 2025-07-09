<script setup>
import { Avatar, TagItem } from '@modrinth/ui'
import { DownloadIcon, HeartIcon, TagIcon } from '@modrinth/assets'
import { formatNumber, formatCategory } from '@modrinth/utils'
import { computed } from 'vue'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { useRouter } from 'vue-router'

dayjs.extend(relativeTime)

const router = useRouter()

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {}
    },
  },
})

const featuredCategory = computed(() => {
  if (props.project.categories.includes('optimization')) {
    return 'optimization'
  }

  if (props.project.categories.length > 0) {
    return props.project.categories[0]
  }
  return undefined
})

const toColor = computed(() => {
  let color = props.project.color

  color >>>= 0
  const b = color & 0xff
  const g = (color >>> 8) & 0xff
  const r = (color >>> 16) & 0xff
  return 'rgba(' + [r, g, b, 1].join(',') + ')'
})

const toTransparent = computed(() => {
  let color = props.project.color

  color >>>= 0
  const b = color & 0xff
  const g = (color >>> 8) & 0xff
  const r = (color >>> 16) & 0xff
  return (
    'linear-gradient(rgba(' +
    [r, g, b, 0.03].join(',') +
    '), 65%, rgba(' +
    [r, g, b, 0.3].join(',') +
    '))'
  )
})
</script>

<template>
  <div
    class="card-shadow bg-bg-raised rounded-xl overflow-clip cursor-pointer hover:brightness-90 transition-all"
    @click="router.push(`/project/${project.slug}`)"
  >
    <div
      class="w-full aspect-[2/1] bg-cover bg-center bg-no-repeat"
      :style="{
        'background-color': project.featured_gallery ?? project.gallery[0] ? null : toColor,
        'background-image': `url(${
          project.featured_gallery ??
          project.gallery[0] ??
          'https://launcher-files.modrinth.com/assets/maze-bg.png'
        })`,
      }"
    >
      <div
        class="badges-wrapper"
        :class="{
          'no-image': !project.featured_gallery && !project.gallery[0],
        }"
        :style="{
          background: !project.featured_gallery && !project.gallery[0] ? toTransparent : null,
        }"
      ></div>
    </div>
    <div class="flex flex-col justify-center gap-2 px-4 py-3">
      <div class="flex gap-2 items-center">
        <Avatar size="48px" :src="project.icon_url" />
        <div class="h-full flex items-center font-bold text-contrast leading-normal">
          <span class="line-clamp-2">{{ project.title }}</span>
        </div>
      </div>
      <p class="m-0 text-sm font-medium line-clamp-3 leading-tight h-[3.25rem]">
        {{ project.description }}
      </p>
      <div class="flex items-center gap-2 text-sm text-secondary font-semibold mt-auto">
        <div
          class="flex items-center gap-1 pr-2 border-0 border-r-[1px] border-solid border-button-border"
        >
          <DownloadIcon />
          {{ formatNumber(project.downloads) }}
        </div>
        <div
          class="flex items-center gap-1 pr-2 border-0 border-r-[1px] border-solid border-button-border"
        >
          <HeartIcon />
          {{ formatNumber(project.follows) }}
        </div>
        <div class="flex items-center gap-1 pr-2">
          <TagIcon />
          <TagItem>
            {{ formatCategory(featuredCategory) }}
          </TagItem>
        </div>
      </div>
    </div>
  </div>
</template>

<style scoped lang="scss"></style>
