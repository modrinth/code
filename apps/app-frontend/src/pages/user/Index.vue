<template>
  <Teleport to="#sidebar-teleport-target">
    <UserSidebarOrganizations :organizations="organizations" :link="(org: Organization) => `/organization/${org.id}${instanceQueryAppendage}`" class="project-sidebar-section" />
    <UserSidebarBadges
      v-if="user"
      :user="user"
      :download-count="sumDownloads"
      class="project-sidebar-section"
    />
    <UserSidebarCollections :collections="collections" :link="(collection: Collection) => `/collection/${collection.id}${instanceQueryAppendage}`" class="project-sidebar-section" />
  </Teleport>
  <div v-if="user" class="p-6 flex flex-col gap-4">
    <InstanceIndicator :instance="instance" />
    <UserHeader :user="user" :project-count="projects.length" :download-count="sumDownloads">
      <template #actions>
        <ButtonStyled circular type="transparent" size="large">
          <OverflowMenu
            :options="[
                  { id: 'report', link: `https://modrinth.com/report?item=user&itemID=${user.id}`, color: 'red' },
                  { id: 'copy-id', action: () => copyId(), shown: themeStore.devMode },
                ]"
            aria-label="More options"
          >
            <MoreVerticalIcon aria-hidden="true" />
            <template #report>
              <ReportIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.reportButton) }}
            </template>
            <template #copy-id>
              <ClipboardCopyIcon aria-hidden="true" />
              {{ formatMessage(commonMessages.copyIdButton) }}
            </template>
          </OverflowMenu>
        </ButtonStyled>
      </template>
    </UserHeader>
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
import { ref, type Ref, watch, computed } from 'vue'
import { handleError } from '@/store/notifications.js'
import {
  ProjectsList,
  UserSidebarOrganizations,
  ButtonStyled,
  commonMessages,
  OverflowMenu,
  UserHeader,
  UserSidebarBadges, UserSidebarCollections
} from '@modrinth/ui'
import { ReportIcon, ClipboardCopyIcon, MoreVerticalIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'
import { useFetch } from '@/helpers/fetch'
import type { User, Project, Organization, Collection } from '@modrinth/utils'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/theme'
import { useInstanceContext } from '@/composables/instance-context'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import ProjectCardActions from '@/components/ui/ProjectCardActions.vue'

const breadcrumbs = useBreadcrumbs()
const route = useRoute()
const { formatMessage } = useVIntl()

const user: Ref<User | null> = ref(null)
const projects: Ref<Project[]> = ref([])
const organizations: Ref<Organization[]> = ref([])
const collections: Ref<Collection[]> = ref([])

async function fetchUser() {
  [ user.value, projects.value, organizations.value, collections.value ] = await Promise.all([
    useFetch(`https://api.modrinth.com/v2/user/${route.params.id}`).catch(handleError),
    useFetch(`https://api.modrinth.com/v2/user/${route.params.id}/projects`).catch(handleError),
    useFetch(`https://api.modrinth.com/v3/user/${route.params.id}/organizations`).catch(handleError),
    useFetch(`https://api.modrinth.com/v3/user/${route.params.id}/collections`).catch(handleError)
  ])

  if (!user.value) {
    return;
  }

  breadcrumbs.setContext({ name: 'User', link: `/user/${user.value.username}` })
  breadcrumbs.setName('User', user.value.username)
}

await fetchUser()

const { instance, instanceContent, instanceQueryAppendage } = await useInstanceContext()

watch(
  () => route.params.id,
  async () => {
    if (route.params.id && route.path.startsWith('/user')) {
      await fetchUser()
    }
  },
)

const themeStore = useTheming()


async function copyId() {
  if (user.value) {
    await navigator.clipboard.writeText(String(user.value.id));
  }
}

const sumDownloads = computed(() => {
  let sum = 0;

  for (const project of projects.value) {
    sum += project.downloads;
  }

  return sum;
});
</script>
<style scoped lang="scss">
.project-sidebar-section {
  @apply p-4 flex flex-col gap-2 border-0 border-[--brand-gradient-border] border-solid;
}
.project-sidebar-section:not(:last-child) {
  @apply border-b-[1px];
}
</style>
