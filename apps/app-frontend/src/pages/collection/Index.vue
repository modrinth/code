<template>
  <Teleport to="#sidebar-teleport-target">
    <CollectionSidebarDescription v-if="collection" :collection="collection" class="project-sidebar-section" />
    <CollectionSidebarCurator v-if="curator" :user="curator" :link="`/user/${curator.id}`" class="project-sidebar-section" />
    <CollectionSidebarDetails v-if="collection" :collection="collection" class="project-sidebar-section" />
  </Teleport>
  <div v-if="collection" class="p-6 flex flex-col gap-4">
    <InstanceIndicator :instance="instance" />
    <CollectionHeader :collection="collection">
      <template #actions>
        <ButtonStyled v-if="themeStore.devMode" circular type="transparent" size="large">
          <OverflowMenu
            :options="[
                  { id: 'copy-id', action: () => copyId(), shown: themeStore.devMode },
                ]"
            aria-label="More options"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #copy-id>
              <ClipboardCopyIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.copyIdButton) }}
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </template>
    </CollectionHeader>
    <div v-if="projects">
      <ProjectsList :projects="projects" :project-link="(project) => `/project/${project.id}${instanceQueryAppendage}`" :experimental-colors="themeStore.featureFlags.project_card_background">
        <template #project-actions="{ project }">
          <ProjectCardActions :instance="instance" :instance-content="instanceContent" :project="project" />
        </template>
      </ProjectsList>
    </div>
  </div>
</template>

<script setup lang="ts">
import { useRoute } from 'vue-router'
import { ref, type Ref, watch } from 'vue'
import { handleError } from '@/store/notifications.js'
import {
  ProjectsList,
  ButtonStyled,
  commonMessages,
  OverflowMenu,
  CollectionHeader, CollectionSidebarCurator, CollectionSidebarDescription, CollectionSidebarDetails
} from '@modrinth/ui'
import { ClipboardCopyIcon, MoreVerticalIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'
import { useFetch } from '@/helpers/fetch'
import type { User, Project, Collection } from '@modrinth/utils'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/theme'
import { useInstanceContext } from '@/composables/instance-context'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import ProjectCardActions from '@/components/ui/ProjectCardActions.vue'

const breadcrumbs = useBreadcrumbs()
const route = useRoute()
const { formatMessage } = useVIntl()

const collection: Ref<Collection | null> = ref(null)
const curator: Ref<User | null> = ref(null)
const projects: Ref<Project[]> = ref([])

async function fetchCollection() {
  collection.value = await useFetch(`https://api.modrinth.com/v3/collection/${route.params.id}`).catch(handleError)

  if (!collection.value) {
    return;
  }

  [ projects.value, curator.value ] = await Promise.all([
    useFetch(`https://api.modrinth.com/v2/projects?ids=${encodeURIComponent(JSON.stringify(collection.value.projects))}`),
    useFetch(`https://api.modrinth.com/v2/user/${collection.value.user}`).catch(handleError),
  ])

  breadcrumbs.setContext({ name: 'Collection', link: `/collection/${collection.value.name}` })
  breadcrumbs.setName('Collection', collection.value.name)
}

await fetchCollection()

const { instance, instanceContent, instanceQueryAppendage } = await useInstanceContext()

watch(
  () => route.params.id,
  async () => {
    if (route.params.id && route.path.startsWith('/collection')) {
      await fetchCollection()
    }
  },
)

const themeStore = useTheming()


async function copyId() {
  if (collection.value) {
    await navigator.clipboard.writeText(String(collection.value.id));
  }
}
</script>
<style scoped lang="scss">
.project-sidebar-section {
  @apply p-4 flex flex-col gap-2 border-0 border-[--brand-gradient-border] border-solid;
}
.project-sidebar-section:not(:last-child) {
  @apply border-b-[1px];
}
</style>
