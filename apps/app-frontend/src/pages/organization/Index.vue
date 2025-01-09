<template>
  <Teleport to="#sidebar-teleport-target">
    <OrganizationSidebarMembers v-if="organization" :members="organization.members" :user-link="(user) => `/user/${user.id}`" class="project-sidebar-section" />
  </Teleport>
  <div v-if="organization" class="flex flex-col gap-4 p-6">
    <InstanceIndicator :instance="instance" />
    <OrganizationHeader :organization="organization" :download-count="sumDownloads" :project-count="projects.length">
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
    </OrganizationHeader>
    <div v-if="projects">
      <ProjectsList :projects="projects" :project-link="(project) => `/project/${project.id}${instanceQueryAppendage}`" :experimental-colors="themeStore.featureFlags.project_card_background">
        <template #project-actions="{ project }">
          <ProjectCardActions :project="project" :instance="instance" :instance-content="instanceContent" />
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
  ButtonStyled,
  commonMessages,
  OverflowMenu,
  OrganizationHeader, OrganizationSidebarMembers
} from '@modrinth/ui'
import { ClipboardCopyIcon, MoreVerticalIcon, DownloadIcon, HeartIcon, BookmarkIcon } from '@modrinth/assets'
import { useVIntl } from '@vintl/vintl'
import { useFetch } from '@/helpers/fetch'
import type { Project, Organization, ProjectV3, Environment } from '@modrinth/utils'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { useTheming } from '@/store/theme'
import { useInstanceContext } from '@/composables/instance-context.ts'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import ProjectCardActions from '@/components/ui/ProjectCardActions.vue'

const breadcrumbs = useBreadcrumbs()
const route = useRoute()
const { formatMessage } = useVIntl()

const organization: Ref<Organization | null> = ref(null)
const projects: Ref<Project[]> = ref([])

const { instance, instanceContent, instanceQueryAppendage } = await useInstanceContext()

async function fetchOrganization() {
  organization.value = await useFetch(`https://api.modrinth.com/v3/organization/${route.params.id}`).catch(handleError)
  projects.value = (await useFetch(`https://api.modrinth.com/v3/organization/${route.params.id}/projects`).catch(handleError)).map((projectV3: ProjectV3) => {
    let type = projectV3.project_types[0]

    if (type === 'plugin' || type === 'datapack') {
      type = 'mod'
    }

    let clientSide: Environment = 'unknown';
    let serverSide: Environment = 'unknown';

    const singleplayer = projectV3.singleplayer && projectV3.singleplayer[0];
    const clientAndServer = projectV3.client_and_server && projectV3.client_and_server[0];
    const clientOnly = projectV3.client_only && projectV3.client_only[0];
    const serverOnly = projectV3.server_only && projectV3.server_only[0];

    // quick and dirty hack to show envs as legacy
    if (singleplayer && clientAndServer && !clientOnly && !serverOnly) {
      clientSide = "required";
      serverSide = "required";
    } else if (singleplayer && clientAndServer && clientOnly && !serverOnly) {
      clientSide = "required";
      serverSide = "unsupported";
    } else if (singleplayer && clientAndServer && !clientOnly && serverOnly) {
      clientSide = "unsupported";
      serverSide = "required";
    } else if (singleplayer && clientAndServer && clientOnly && serverOnly) {
      clientSide = "optional";
      serverSide = "optional";
    }

    const projectV2: Project = {
      ...projectV3,
      title: projectV3.name,
      description: projectV3.summary,
      body: projectV3.description,
      project_type: type,
      team: projectV3.team_id,
      donation_urls: [],
      client_side: clientSide,
      server_side: serverSide,
    }
    return projectV2
  })

  if (!organization.value) {
    return;
  }

  breadcrumbs.setName('Organization', organization.value.name)
}

await fetchOrganization()

watch(
  () => route.params.id,
  async () => {
    if (route.params.id && route.path.startsWith('/organization')) {
      await fetchOrganization()
    }
  },
)

const themeStore = useTheming()

async function copyId() {
  if (organization.value) {
    await navigator.clipboard.writeText(String(organization.value.id));
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
