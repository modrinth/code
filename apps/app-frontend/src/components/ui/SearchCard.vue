<template>
  <Card
    class="card button-base"
    @click="
      () => {
        emits('open')
        $router.push({
          path: `/project/${project.project_id ?? project.id}/`,
          query: { i: props.instance ? props.instance.path : undefined },
        })
      }
    "
  >
    <div class="icon">
      <Avatar :src="project.icon_url" size="md" class="search-icon" />
    </div>
    <div class="content-wrapper">
      <div class="title joined-text">
        <h2>{{ project.title }}</h2>
        <span v-if="project.author">by {{ project.author }}</span>
      </div>
      <div class="description">
        {{ project.description }}
      </div>
      <div class="tags">
        <Categories :categories="categories" :type="project.project_type">
          <EnvironmentIndicator
            :type-only="project.moderation"
            :client-side="project.client_side"
            :server-side="project.server_side"
            :type="project.project_type"
            :search="true"
          />
        </Categories>
      </div>
    </div>
    <div class="stats button-group">
      <div v-if="featured" class="badge">
        <StarIcon />
        Featured
      </div>
      <div class="badge">
        <DownloadIcon />
        {{ formatNumber(project.downloads) }}
      </div>
      <div class="badge">
        <HeartIcon />
        {{ formatNumber(project.follows ?? project.followers) }}
      </div>
      <div class="badge">
        <CalendarIcon />
        {{ formatCategory(dayjs(project.date_modified ?? project.updated).fromNow()) }}
      </div>
    </div>
    <div v-if="project.author" class="install">
      <Button color="primary" :disabled="installed || installing" @click.stop="install()">
        <DownloadIcon v-if="!installed" />
        <CheckIcon v-else />
        {{ installing ? 'Installing' : installed ? 'Installed' : 'Install' }}
      </Button>
    </div>
  </Card>
</template>

<script setup>
import { DownloadIcon, HeartIcon, CalendarIcon, CheckIcon, StarIcon } from '@modrinth/assets'
import { Avatar, Card, Categories, EnvironmentIndicator, Button } from '@modrinth/ui'
import { formatNumber, formatCategory } from '@modrinth/utils'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref } from 'vue'
import { install as installVersion } from '@/store/install.js'
dayjs.extend(relativeTime)

const props = defineProps({
  backgroundImage: {
    type: String,
    default: null,
  },
  project: {
    type: Object,
    required: true,
  },
  categories: {
    type: Array,
    required: true,
  },
  instance: {
    type: Object,
    default: null,
  },
  featured: {
    type: Boolean,
    default: false,
  },
  installed: {
    type: Boolean,
    default: false,
  },
})

const emits = defineEmits(['open'])

const installing = ref(false)
const installed = ref(props.installed)

async function install() {
  installing.value = true
  await installVersion(
    props.project.project_id,
    null,
    props.instance ? props.instance.path : null,
    'SearchCard',
    (version) => {
      installing.value = false

      if (props.instance && version) {
        installed.value = true
      }
    },
  )
}
</script>

<style scoped lang="scss">
.icon {
  grid-column: 1;
  grid-row: 1;
  align-self: center;
  height: 6rem;
}

.content-wrapper {
  display: flex;
  justify-content: space-between;
  grid-column: 2 / 4;
  flex-direction: column;
  grid-row: 1;
  gap: 0.5rem;

  .description {
    word-wrap: break-word;
    overflow-wrap: anywhere;
  }
}

.stats {
  grid-column: 1 / 3;
  grid-row: 2;
  justify-self: stretch;
  align-self: start;
}

.install {
  grid-column: 3 / 4;
  grid-row: 2;
  justify-self: end;
  align-self: start;
}

.card {
  margin-bottom: 0;
  display: grid;
  grid-template-columns: 6rem auto 7rem;
  gap: 0.75rem;
  padding: 1rem;

  &:active:not(&:disabled) {
    scale: 0.98 !important;
  }
}

.joined-text {
  display: inline-flex;
  flex-wrap: wrap;
  flex-direction: row;
  column-gap: 0.5rem;
  align-items: baseline;
  overflow: hidden;
  text-overflow: ellipsis;

  h2 {
    margin-bottom: 0 !important;
    word-wrap: break-word;
    overflow-wrap: anywhere;
  }
}

.button-group {
  display: inline-flex;
  flex-direction: row;
  gap: 0.5rem;
  align-items: flex-start;
  flex-wrap: wrap;
  justify-content: flex-start;
}
</style>
