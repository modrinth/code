<template>
  <Card class="card button-base" @click="$router.push(`/project/${project.project_id}/`)">
    <div id="icon">
      <Avatar :src="project.icon_url" size="md" class="search-icon" />
    </div>
    <div id="title" class="no-wrap joined-text">
      <h2>{{ project.title }}</h2>
      <span>by {{ project.author }}</span>
    </div>
    <div id="description">
      {{ project.description }}
      <Categories :categories="categories" :type="project.project_type" class="tags">
        <EnvironmentIndicator
          :type-only="project.moderation"
          :client-side="project.client_side"
          :server-side="project.server_side"
          :type="project.project_type"
          :search="true"
        />
      </Categories>
    </div>
    <div id="stats" class="button-group">
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
    <div id="install">
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
import {onMounted, ref} from 'vue'
import {add_project_from_version as installMod, check_installed, list} from '@/helpers/profile.js'
import { install as packInstall } from '@/helpers/pack.js'
import { installVersionDependencies } from '@/helpers/utils.js'
import { ofetch } from 'ofetch'
import { useRouter } from 'vue-router'
dayjs.extend(relativeTime)

const router = useRouter()

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
const installed = ref(false)

onMounted(async () => {
  installed.value = props.instance && await check_installed(props.instance.path, props.project.project_id)
})

const markInstalled = () => {
  installed.value = true
}

const install = async () => {
  installing.value = true
  const versions = await ofetch(
    `https://api.modrinth.com/v2/project/${props.project.project_id}/version`
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
    const packs = Object.values(await list())
    if (
      packs.length === 0 ||
      !packs
        .map((value) => value.metadata)
        .find((pack) => pack.linked_data?.project_id === props.project.project_id)
    ) {
      let id = await packInstall(queuedVersionData.id)
      await router.push({ path: `/instance/${encodeURIComponent(id)}` })
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
          markInstalled
        )
        installing.value = false
        return
      } else {
        await installMod(props.instance.path, queuedVersionData.id)
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
#icon {
  grid-column: 1;
  grid-row: 1 / 3;
  align-self: center;
}

#title {
  grid-column: 2 / 4;
  grid-row: 1;
  align-self: start;
}

#description {
  grid-column: 2 / 4;
  grid-row: 2;
  display: flex;
  flex-direction: column;
  justify-content: space-between;
  gap: 0.5rem;
  align-self: stretch;
  justify-self: start;
}

#stats {
  grid-column: 1 / 3;
  grid-row: 3;
  justify-self: stretch;
  align-self: start;
}

#install {
  grid-column: 3 / 4;
  grid-row: 3;
  justify-self: end;
  align-self: start;
}

.card {
  margin-bottom: 0;
  display: grid;
  grid-template-columns: 6rem auto 7rem;
  grid-template-rows: min-content auto auto;
  gap: 1rem;
  padding: 1rem;

  &:active:not(&:disabled) {
    scale: 0.98 !important;
  }
}

.background {
  position: absolute;
  width: 100%;
  height: 100%;
  border-radius: var(--radius-lg);
  overflow: hidden;
  display: flex;
  align-items: center;
  justify-content: center;
  opacity: 0.15;
  z-index: -1;
  background-image: linear-gradient(to right, rgba(0, 0, 0, 0), var(--color-raised-bg));
}

.background-img {
  position: absolute;
  top: 0;
  left: 0;
  width: 100%;
  height: 100%;
  border-radius: var(--radius-lg);
  object-fit: cover;
}

.joined-text {
  display: inline-flex;
  flex-wrap: wrap;
  flex-direction: row;
  column-gap: 0.5rem;
  align-items: baseline;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;

  h2 {
    margin-bottom: 0 !important;
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
