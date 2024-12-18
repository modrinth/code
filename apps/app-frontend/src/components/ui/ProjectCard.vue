<script setup>
import { Card, Avatar, Button, TagItem } from '@modrinth/ui'
import { DownloadIcon, HeartIcon, CalendarIcon, TagIcon } from '@modrinth/assets'
import { formatNumber, formatCategory } from '@modrinth/utils'
import { computed, ref } from 'vue'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { useRouter } from 'vue-router'
import { install as installVersion } from '@/store/install.js'

dayjs.extend(relativeTime)

const router = useRouter()
const installing = ref(false)

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

const newStyle = ref(true)

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

const install = async (e) => {
  e?.stopPropagation()
  installing.value = true
  await installVersion(
    props.project.project_id,
    null,
    props.instance ? props.instance.path : null,
    'ProjectCard',
    () => {
      installing.value = false
    },
  )
}
</script>

<template>
  <template v-if="newStyle">
    <div class="card-shadow bg-bg-raised rounded-xl overflow-clip cursor-pointer active:scale-[0.98] transition-transform" @click="router.push(`/project/${project.slug}`)">
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
          <Avatar
            size="48px"
            :src="project.icon_url"
          />
          <div class="h-full flex items-center font-bold text-contrast leading-normal">
            <span class="line-clamp-2">{{ project.title }}</span>
          </div>
        </div>
        <p class="m-0 text-sm font-medium line-clamp-3 leading-tight h-[3.25rem]">
          {{ project.description }}
        </p>
        <div class="flex items-center gap-2 text-sm text-secondary font-semibold mt-auto">
          <div class="flex items-center gap-1 pr-2 border-0 border-r-[1px] border-solid border-button-border">
            <DownloadIcon />
            {{ formatNumber(project.downloads) }}
          </div>
          <div class="flex items-center gap-1 pr-2 border-0 border-r-[1px] border-solid border-button-border">
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
  <div v-else class="wrapper">
    <Card class="project-card button-base" @click="router.push(`/project/${project.slug}`)">
      <div
        class="banner"
        :style="{
          'background-color': project.featured_gallery ?? project.gallery[0] ? null : toColor,
          'background-image': `url(${
            project.featured_gallery ??
            project.gallery[0] ??
            'https://launcher-files.modrinth.com/assets/maze-bg.png'
          })`,
          'no-image': !project.featured_gallery && !project.gallery[0],
        }"
      >
        <div class="badges">
          <div class="badge">
            <DownloadIcon />
            {{ formatNumber(project.downloads) }}
          </div>
          <div class="badge">
            <HeartIcon />
            {{ formatNumber(project.follows) }}
          </div>
          <div class="badge">
            <CalendarIcon />
            {{ formatCategory(dayjs(project.date_modified).fromNow()) }}
          </div>
        </div>
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
      <Avatar class="icon" size="sm" :src="project.icon_url" />
      <div class="title">
        <div class="title-text">
          {{ project.title }}
        </div>
        <div class="author">by {{ project.author }}</div>
      </div>
      <div class="description">
        {{ project.description }}
      </div>
    </Card>
    <Button color="primary" class="install" :disabled="installing" @click="install">
      <DownloadIcon />
      {{ installing ? 'Installing' : 'Install' }}
    </Button>
  </div>
</template>

<style scoped lang="scss">
.wrapper {
  position: relative;
  aspect-ratio: 1;

  &:hover {
    .install:enabled {
      opacity: 1;
    }
  }
}

.project-card {
  display: grid;
  grid-gap: 1rem;
  grid-template:
    '. . . .' 0
    '. icon title .' 3rem
    'banner banner banner banner' auto
    '. description description .' 3.5rem
    '. . . .' 0 / 0 3rem minmax(0, 1fr) 0;
  max-width: 100%;
  height: 100%;
  padding: 0;
  margin: 0;

  .icon {
    grid-area: icon;
  }

  .title {
    max-width: 100%;
    display: flex;
    flex-direction: column;
    justify-content: center;
    grid-area: title;
    white-space: nowrap;

    .title-text {
      width: 100%;
      overflow: hidden;
      text-overflow: ellipsis;
      font-size: var(--font-size-md);
      font-weight: bold;
    }
  }

  .author {
    font-size: var(--font-size-sm);
    grid-area: author;
  }

  .banner {
    grid-area: banner;
    background-size: cover;
    background-position: center;
    position: relative;

    .badges-wrapper {
      width: 100%;
      height: 100%;
      display: flex;
      position: absolute;
      top: 0;
      left: 0;
      mix-blend-mode: hard-light;
    }

    .badges {
      position: absolute;
      top: 0;
      left: 0;
      width: 100%;
      height: 100%;
      padding: var(--gap-sm);
      gap: var(--gap-xs);
      display: flex;
      z-index: 10;
      flex-direction: row;
      justify-content: flex-end;
      align-items: flex-end;
    }
  }

  .description {
    grid-area: description;
    overflow: hidden;
    text-overflow: ellipsis;
    display: -webkit-box;
    -webkit-line-clamp: 3;
    -webkit-box-orient: vertical;
  }
}

.badge {
  background-color: var(--color-raised-bg);
  font-size: var(--font-size-xs);
  padding: var(--gap-xs) var(--gap-sm);
  border-radius: var(--radius-sm);

  svg {
    width: 1rem;
    height: 1rem;
    margin-right: var(--gap-xs);
  }
}

.install {
  position: absolute;
  top: calc(5rem + var(--gap-sm));
  right: var(--gap-sm);
  z-index: 10;
  opacity: 0;
  transition: opacity 0.2s ease-in-out;

  svg {
    width: 1rem;
    height: 1rem;
  }
}
</style>
