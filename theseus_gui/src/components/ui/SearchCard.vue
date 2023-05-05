<template>
  <Card class="card button-base" @click="$router.push(`/project/${project.project_id}/`)">
    <div class="card-content">
      <div class="content">
        <Avatar :src="project.icon_url" size="md" />
      </div>
      <div class="content description">
        <div class="content">
          <div class="no-wrap joined-text">
            <h2>{{ project.title }}</h2>
            <span>by {{ project.author }}</span>
          </div>
          <p>
            {{ project.description }}
          </p>
        </div>
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
      <div class="content badges">
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
        <div class="content">
          <div class="button-group">
            <div class="badge">
              <HeartIcon />
              {{ formatNumber(project.follows) }}
            </div>
            <div class="badge">
              <DownloadIcon />
              {{ formatNumber(project.downloads) }}
            </div>
            <div class="badge">
              <CalendarIcon />
              {{ formatCategory(dayjs(project.date_modified).fromNow()) }}
            </div>
          </div>
        </div>
      </div>
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
} from 'omorphia'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ref } from 'vue'
import { add_project_from_version as installMod, list } from '@/helpers/profile.js'
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
})

const installed = ref(
  props.instance &&
    Object.values(props.instance.projects).some(
      (project) => project?.metadata?.project?.id === props.project.project_id
    )
)
const installing = ref(false)

const install = async () => {
  installing.value = true
  const versions = await ofetch(
    `https://api.modrinth.com/v2/project/${props.project.project_id}/version`
  )
  let queuedVersionData

  if (!props.instance) {
    queuedVersionData = versions[0]
  } else {
    queuedVersionData = versions.find((v) => v.game_versions.includes(props.instance.game_version))
  }

  console.log(versions)
  console.log(queuedVersionData)

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
      await installMod(props.instance.path, queuedVersionData.id)
      installVersionDependencies(props.instance, queuedVersionData)
    } else {
      props.modInstallModal.show(props.project.project_id, versions)
    }
    if (props.instance) installed.value = true
  }

  installing.value = false
}
</script>

<style scoped lang="scss">
.card {
  padding: 0;
  z-index: 0;
  margin-bottom: 0;

  &:active:not(&:disabled) {
    scale: 0.98 !important;
  }
}

.card-content {
  display: flex;
  flex-direction: row;
  align-items: stretch;
  gap: 1rem;
  padding: var(--gap-xl);
  height: 100%;
}

.content {
  display: flex;
  flex-direction: column;
  gap: 0.5rem;
  justify-content: center;
}

.description {
  flex-grow: 1;
  justify-content: space-between;
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
  width: fit-content;
  height: fit-content;
  border-radius: var(--radius-md);
  white-space: nowrap;
  font-weight: 500;
  align-items: center;

  svg {
    width: 1.1rem;
    height: 1.1rem;
    margin-right: 0.5rem;
  }
}

.badges {
  align-items: flex-end;
  justify-content: space-between;
}

.button-group {
  display: inline-flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 0.5rem;
  align-items: center;
  justify-content: flex-end;
}
</style>
