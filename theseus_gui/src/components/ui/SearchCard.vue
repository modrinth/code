<template>
  <Card class="card button-base" @click="$router.push(`/project/${project.project_id}/`)">
    <div class="icon">
      <Avatar :src="project.icon_url" size="md" class="search-icon" />
    </div>
    <div class="content-wrapper">
      <div class="title joined-text">
        <h2>{{ project.title }}</h2>
        <span>by {{ project.author }}</span>
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
        {{ formatNumber(project.follows) }}
      </div>
      <div class="badge">
        <CalendarIcon />
        {{ formatCategory(dayjs(project.date_modified).fromNow()) }}
      </div>
    </div>
    <div class="install">
      <Button
        :to="`/browse/${project.slug}`"
        color="primary"
        :disabled="installed || installing"
        @click.stop="install()"
      >
        <DownloadIcon v-if="!installed" />
        <CheckIcon v-else />
        {{ installing ? 'Installing' : installed ? 'Installed' : 'Install' }}
      </Button>
    </div>
  </Card>
</template>

<script setup>
import {
  Avatar,
  Card,
  Categories,
  EnvironmentIndicator,
  Button,
  DownloadIcon,
  formatNumber,
  formatCategory,
  HeartIcon,
  CalendarIcon,
  CheckIcon,
  StarIcon,
} from 'omorphia'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref } from 'vue'
import { add_project_from_version as installMod, list } from '@/helpers/profile.js'
import { install as packInstall } from '@/helpers/pack.js'
import { installVersionDependencies } from '@/helpers/utils.js'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/notifications.js'
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
  confirmModal: {
    type: Object,
    default: null,
  },
  modInstallModal: {
    type: Object,
    default: null,
  },
  incompatibilityWarningModal: {
    type: Object,
    default: null,
  },
  featured: {
    type: Boolean,
    default: false,
  },
})

const installing = ref(false)

const installed = ref(
  props.instance
    ? Object.values(props.instance.projects).some(
        (p) => p.metadata?.project?.id === props.project.project_id
      )
    : false
)

async function install() {
  installing.value = true
  const versions = await useFetch(
    `https://api.modrinth.com/v2/project/${props.project.project_id}/version`,
    'project versions'
  )
  let queuedVersionData

  if (!props.instance) {
    queuedVersionData = versions[0]
  } else {
    queuedVersionData = versions.find(
      (v) =>
        v.game_versions.includes(props.instance.metadata.game_version) &&
        v.loaders.includes(props.instance.metadata.loader)
    )
  }

  if (props.project.project_type === 'modpack') {
    const packs = Object.values(await list().catch(handleError))
    if (
      packs.length === 0 ||
      !packs
        .map((value) => value.metadata)
        .find((pack) => pack.linked_data?.project_id === props.project.project_id)
    ) {
      await packInstall(queuedVersionData.id, props.project.title, props.project.icon_url).catch(
        handleError
      )
    } else {
      props.confirmModal.show(queuedVersionData.id)
    }
  } else {
    if (props.instance) {
      if (!queuedVersionData) {
        props.incompatibilityWarningModal.show(
          props.instance,
          props.project.title,
          versions,
          () => (installed.value = true)
        )
        installing.value = false
        return
      } else {
        await installMod(props.instance.path, queuedVersionData.id).catch(handleError)
        installVersionDependencies(props.instance, queuedVersionData)
      }
    } else {
      props.modInstallModal.show(props.project.project_id, versions)
      installing.value = false
      return
    }
    if (props.instance) installed.value = true
  }

  installing.value = false
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

.badge {
  display: flex;
  border-radius: var(--radius-md);
  white-space: nowrap;
  font-weight: 500;
  align-items: center;
  background-color: var(--color-bg);
  padding-block: var(--gap-sm);
  padding-inline: var(--gap-lg);

  svg {
    width: 1.1rem;
    height: 1.1rem;
    margin-right: 0.5rem;
  }

  &.featured {
    background-color: var(--color-brand-highlight);
    color: var(--color-contrast);
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
