<script setup>
import {
  DownloadIcon,
  ChevronRightIcon,
  formatNumber,
  CalendarIcon,
  HeartIcon,
  Avatar,
  Card,
} from 'omorphia'
import { onMounted, onUnmounted, ref } from 'vue'

const modsRow = ref(null)
const rows = ref(null)
const maxInstancesPerRow = ref(0)
const maxProjectsPerRow = ref(0)

const calculateCardsPerRow = () => {
  // Calculate how many cards fit in one row
  const containerWidth = rows.value[0].clientWidth
  // Convert container width from pixels to rem
  const containerWidthInRem =
    containerWidth / parseFloat(getComputedStyle(document.documentElement).fontSize)
  maxInstancesPerRow.value = Math.floor((containerWidthInRem + 1) / 11)
  maxProjectsPerRow.value = Math.floor((containerWidthInRem + 1) / 17)
}

onMounted(() => {
  calculateCardsPerRow()
  window.addEventListener('resize', calculateCardsPerRow)
})

onUnmounted(() => {
  window.removeEventListener('resize', calculateCardsPerRow)
})

defineProps({
  showInstance: {
    type: Boolean,
    default: false,
  },
})
</script>

<template>
  <div class="content">
    <div
      v-for="(row, index) in ['Jump back in', 'Popular modpacks', 'Popular mods']"
      ref="rows"
      :key="row"
      class="row"
    >
      <div class="header">
        <p>{{ row }}</p>
        <ChevronRightIcon />
      </div>
      <section v-if="index < 1" ref="modsRow" class="instances">
        <Card
          v-for="project in maxInstancesPerRow"
          :key="project"
          class="instance-card-item button-base"
          :class="{ highlighted: showInstance }"
        >
          <Avatar
            size="sm"
            src="https://launcher-files.modrinth.com/assets/default_profile.png"
            alt="Mod card"
            class="mod-image"
          />
          <div class="project-info">
            <p class="title">Example Profile</p>
            <p class="description">Forge/Fabric 1.20.1</p>
          </div>
        </Card>
      </section>
      <section v-else ref="modsRow" class="projects">
        <div v-for="project in maxProjectsPerRow" :key="project" class="wrapper">
          <Card class="project-card button-base" :class="{ highlighted: showInstance }">
            <div
              class="banner no-image"
              :style="{
                'background-image': `url(https://launcher-files.modrinth.com/assets/maze-bg.png)`,
              }"
            >
              <div class="badges">
                <div class="badge">
                  <DownloadIcon />
                  {{ formatNumber(69420) }}
                </div>
                <div class="badge">
                  <HeartIcon />
                  {{ formatNumber(69) }}
                </div>
                <div class="badge">
                  <CalendarIcon />
                  Today
                </div>
              </div>
              <div
                class="badges-wrapper no-image"
                :style="{
                  background:
                    'linear-gradient(rgba(' +
                    [27, 217, 106, 0.03].join(',') +
                    '), 65%, rgba(' +
                    [27, 217, 106, 0.3].join(',') +
                    '))',
                }"
              ></div>
            </div>
            <Avatar
              class="icon"
              size="sm"
              src="https://launcher-files.modrinth.com/assets/default_profile.png"
            />
            <div class="title">
              <div class="title-text">Example Project</div>
              <div class="author">by Modrinth</div>
            </div>
            <div class="description">
              An example project hangin on the Rinth. Very cool project, its probably on Forge and
              Fabric. Probably has a 401k and a family.
            </div>
          </Card>
        </div>
      </section>
    </div>
  </div>
</template>
<style lang="scss" scoped>
.content {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  width: 100%;
  padding: 1rem;
  gap: 1rem;

  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }
}

.row {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  width: 100%;
  min-width: 100%;

  &:nth-child(even) {
    background: var(--color-bg);
  }

  .header {
    width: 100%;
    margin-bottom: 1rem;
    gap: var(--gap-xs);
    display: flex;
    flex-direction: row;
    align-items: center;

    p {
      margin: 0;
      font-size: var(--font-size-lg);
      font-weight: bolder;
      white-space: nowrap;
      color: var(--color-contrast);
    }

    svg {
      height: 1.5rem;
      width: 1.5rem;
      color: var(--color-contrast);
    }
  }

  .instances {
    display: grid;
    grid-template-columns: repeat(auto-fill, minmax(10rem, 1fr));
    grid-gap: 1rem;
    width: 100%;
  }

  .projects {
    display: grid;
    width: 100%;
    grid-template-columns: repeat(auto-fill, minmax(16rem, 1fr));
    grid-gap: 1rem;

    .item {
      width: 100%;
      max-width: 100%;
    }
  }
}

.loading-indicator {
  width: 2.5rem !important;
  height: 2.5rem !important;

  svg {
    width: 2.5rem !important;
    height: 2.5rem !important;
  }
}

.instance {
  position: relative;
}

.instance-card-item {
  display: flex;
  flex-direction: column;
  align-items: center;
  justify-content: center;
  cursor: pointer;
  padding: var(--gap-md);
  transition: 0.1s ease-in-out all !important; /* overrides Omorphia defaults */
  margin-bottom: 0;

  .mod-image {
    --size: 100%;

    width: 100% !important;
    height: auto !important;
    max-width: unset !important;
    max-height: unset !important;
    aspect-ratio: 1 / 1 !important;
  }

  .project-info {
    margin-top: 1rem;
    width: 100%;

    .title {
      color: var(--color-contrast);
      overflow: hidden;
      white-space: nowrap;
      text-overflow: ellipsis;
      width: 100%;
      margin: 0;
      font-weight: 600;
      font-size: 1rem;
      line-height: 110%;
      display: inline-block;
    }

    .description {
      color: var(--color-base);
      display: -webkit-box;
      -webkit-line-clamp: 2;
      -webkit-box-orient: vertical;
      overflow: hidden;
      font-weight: 500;
      font-size: 0.775rem;
      line-height: 125%;
      margin: 0.25rem 0 0;
      text-transform: capitalize;
      white-space: nowrap;
      text-overflow: ellipsis;
    }
  }
}

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
      z-index: 1;
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
</style>
