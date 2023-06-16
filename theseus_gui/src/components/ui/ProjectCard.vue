<script setup>
import {Card, Avatar, formatNumber, formatCategory, DownloadIcon, HeartIcon, CalendarIcon} from "omorphia";
import {computed} from "vue";
import dayjs from "dayjs";
import relativeTime from 'dayjs/plugin/relativeTime';
dayjs.extend(relativeTime)

const props = defineProps({
  project: {
    type: Object,
    default() {
      return {};
    },
  },
});

const toColor = computed(() => {
  let color = props.project.color

  color >>>= 0
  const b = color & 0xff
  const g = color >>> 8 & 0xff
  const r = color >>> 16 & 0xff
  return 'rgba(' + [r, g, b, 1].join(',') + ')'
})

const toTransparent = computed(() => {
  let color = props.project.color

  color >>>= 0
  const b = color & 0xff
  const g = color >>> 8 & 0xff
  const r = color >>> 16 & 0xff
  return 'linear-gradient(rgba(' + [r, g, b, 0.03].join(',') + '), 65%, rgba(' + [r, g, b, 0.3].join(',') + '))'
})
</script>

<template>
  <Card class="project-card button-base">
    <div
      class="banner"
      :style="{
        'background-color': project.featured_gallery ?? project.gallery[0] ? null : toColor,
        'background-image': `url(${project.featured_gallery ?? project.gallery[0] ?? 'https://cdn.discordapp.com/attachments/817413688771608587/1119143634319724564/image.png'})`,
        'no-image': !project.featured_gallery && !project.gallery[0]
    }"
    >
      <div
        class="badges"
        :class="{
          'no-image': !project.featured_gallery && !project.gallery[0]
        }"
        :style="{
          'background': !project.featured_gallery && !project.gallery[0] ? toTransparent : null,
        }"
      >
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
    </div>
    <Avatar class="icon" size="sm" :src="project.icon_url"/>
    <div class="title">
      <div class="title-text">
        {{ project.title }}
      </div>
      <div class="author">
        by {{ project.author }}
      </div>
    </div>
    <div class="description">
      {{ project.description }}
    </div>
  </Card>
</template>

<style scoped lang="scss">
.project-card {
  display: grid;
  grid-gap: 1rem;
  grid-template:
    ". . . ." 0
    ". icon title ." 3rem
    "banner banner banner banner" auto
    ". description description ." 3.5rem
    ". . . ." 0 / 0 3rem minmax(0, 1fr) 0;
  aspect-ratio: 1;
  max-width: 100%;
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

    .badges {
      padding: var(--gap-sm);
      gap: var(--gap-xs);
      width: 100%;
      height: 100%;
      display: flex;
      flex-direction: row;
      justify-content: flex-end;
      align-items: flex-end;
      mix-blend-mode: hard-light;
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
</style>
