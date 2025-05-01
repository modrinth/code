<template>
  <div>
    <Teleport to="#sidebar-teleport-target">
      <ProjectSidebarCompatibility
        :project="data"
        :tags="{ loaders: allLoaders, gameVersions: allGameVersions }"
        class="project-sidebar-section"
      />
      <ProjectSidebarLinks link-target="_blank" :project="data" class="project-sidebar-section" />
      <ProjectSidebarCreators
        :organization="null"
        :members="members"
        :org-link="(slug) => `https://modrinth.com/organization/${slug}`"
        :user-link="(username) => `https://modrinth.com/user/${username}`"
        link-target="_blank"
        class="project-sidebar-section"
      />
      <ProjectSidebarDetails
        :project="data"
        :has-versions="versions.length > 0"
        :link-target="`_blank`"
        class="project-sidebar-section"
      />
    </Teleport>
    <div class="flex flex-col gap-4 p-6">
      <InstanceIndicator v-if="instance" :instance="instance" />
      <template v-if="data">
        <Teleport
          v-if="themeStore.featureFlags.project_background"
          to="#background-teleport-target"
        >
          <ProjectBackgroundGradient :project="data" />
        </Teleport>
        <ProjectHeader :project="data" @contextmenu.prevent.stop="handleRightClick">
          <template #actions>
            <ButtonStyled size="large" color="brand">
              <button
                v-tooltip="installed ? `This project is already installed` : null"
                :disabled="installed || installing"
                @click="install(null)"
              >
                <DownloadIcon v-if="!installed && !installing" />
                <CheckIcon v-else-if="installed" />
                {{ installing ? 'Installing...' : installed ? 'Installed' : 'Install' }}
              </button>
            </ButtonStyled>
            <ButtonStyled size="large" circular type="transparent">
              <OverflowMenu
                :tooltip="`More options`"
                :options="[
                  {
                    id: 'follow',
                    disabled: true,
                    tooltip: 'Coming soon',
                    action: () => {},
                  },
                  {
                    id: 'save',
                    disabled: true,
                    tooltip: 'Coming soon',
                    action: () => {},
                  },
                  {
                    id: 'open-in-browser',
                    link: `https://modrinth.com/${data.project_type}/${data.slug}`,
                    external: true,
                  },
                  {
                    divider: true,
                  },
                  {
                    id: 'report',
                    color: 'red',
                    hoverFilled: true,
                    link: `https://modrinth.com/report?item=project&itemID=${data.id}`,
                  },
                ]"
                aria-label="More options"
              >
                <MoreVerticalIcon aria-hidden="true" />
                <template #open-in-browser> <ExternalIcon /> Open in browser </template>
                <template #follow> <HeartIcon /> Follow </template>
                <template #save> <BookmarkIcon /> Save </template>
                <template #report> <ReportIcon /> Report </template>
              </OverflowMenu>
            </ButtonStyled>
          </template>
        </ProjectHeader>
        <NavTabs
          :links="[
            {
              label: 'Description',
              href: `/project/${$route.params.id}`,
            },
            {
              label: 'Versions',
              href: {
                path: `/project/${$route.params.id}/versions`,
                query: { l: instance?.loader, g: instance?.game_version },
              },
              subpages: ['version'],
            },
            {
              label: 'Gallery',
              href: `/project/${$route.params.id}/gallery`,
              shown: data.gallery.length > 0,
            },
          ]"
        />
        <RouterView
          :project="data"
          :versions="versions"
          :members="members"
          :instance="instance"
          :install="install"
          :installed="installed"
          :installing="installing"
          :installed-version="installedVersion"
        />
      </template>
      <template v-else> Project data couldn't not be loaded. </template>
    </div>
    <ContextMenu ref="options" @option-clicked="handleOptionsClick">
      <template #install> <DownloadIcon /> Install </template>
      <template #open_link> <GlobeIcon /> Open in Modrinth <ExternalIcon /> </template>
      <template #copy_link> <ClipboardCopyIcon /> Copy link </template>
    </ContextMenu>
  </div>
</template>

<script setup>
import {
  BookmarkIcon,
  MoreVerticalIcon,
  DownloadIcon,
  ReportIcon,
  HeartIcon,
  ExternalIcon,
  CheckIcon,
  GlobeIcon,
  ClipboardCopyIcon,
} from '@modrinth/assets'
import {
  ProjectHeader,
  ProjectSidebarCompatibility,
  ButtonStyled,
  OverflowMenu,
  ProjectSidebarLinks,
  ProjectSidebarCreators,
  ProjectSidebarDetails,
  ProjectBackgroundGradient,
} from '@modrinth/ui'

import { get_categories, get_game_versions, get_loaders } from '@/helpers/tags'
import { get as getInstance, get_projects as getInstanceProjects } from '@/helpers/profile'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { useRoute, useRouter } from 'vue-router'
import { ref, shallowRef, watch } from 'vue'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import { handleError } from '@/store/notifications.js'
import ContextMenu from '@/components/ui/ContextMenu.vue'
import { install as installVersion } from '@/store/install.js'
import { get_project, get_team, get_version_many } from '@/helpers/cache.js'
import NavTabs from '@/components/ui/NavTabs.vue'
import { useTheming } from '@/store/state.js'
import InstanceIndicator from '@/components/ui/InstanceIndicator.vue'
import { openUrl } from '@tauri-apps/plugin-opener'

dayjs.extend(relativeTime)

const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()
const themeStore = useTheming()

const installing = ref(false)
const data = shallowRef(null)
const versions = shallowRef([])
const members = shallowRef([])
const categories = shallowRef([])
const instance = ref(null)
const instanceProjects = ref(null)

const installed = ref(false)
const installedVersion = ref(null)

const [allLoaders, allGameVersions] = await Promise.all([
  get_loaders().catch(handleError).then(ref),
  get_game_versions().catch(handleError).then(ref),
])

async function fetchProjectData() {
  const project = await get_project(route.params.id, 'must_revalidate').catch(handleError)

  if (!project) {
    handleError('Error loading project')
    return
  }

  data.value = project
  ;[versions.value, members.value, categories.value, instance.value, instanceProjects.value] =
    await Promise.all([
      get_version_many(project.versions, 'must_revalidate').catch(handleError),
      get_team(project.team).catch(handleError),
      get_categories().catch(handleError),
      route.query.i ? getInstance(route.query.i).catch(handleError) : Promise.resolve(),
      route.query.i ? getInstanceProjects(route.query.i).catch(handleError) : Promise.resolve(),
    ])

  versions.value = versions.value.sort((a, b) => dayjs(b.date_published) - dayjs(a.date_published))

  if (instanceProjects.value) {
    const installedFile = Object.values(instanceProjects.value).find(
      (x) => x.metadata && x.metadata.project_id === data.value.id,
    )
    if (installedFile) {
      installed.value = true
      installedVersion.value = installedFile.metadata.version_id
    }
  }
  breadcrumbs.setName('Project', data.value.title)
}

await fetchProjectData()

watch(
  () => route.params.id,
  async () => {
    if (route.params.id && route.path.startsWith('/project')) {
      await fetchProjectData()
    }
  },
)

async function install(version) {
  installing.value = true
  await installVersion(
    data.value.id,
    version,
    instance.value ? instance.value.path : null,
    'ProjectPage',
    (version) => {
      installing.value = false

      if (instance.value && version) {
        installed.value = true
        installedVersion.value = version
      }
    },
    (profile) => {
      router.push(`/instance/${profile}`)
    },
  )
}

const options = ref(null)
const handleRightClick = (event) => {
  options.value.showMenu(event, data.value, [
    {
      name: 'install',
    },
    {
      type: 'divider',
    },
    {
      name: 'open_link',
    },
    {
      name: 'copy_link',
    },
  ])
}
const handleOptionsClick = (args) => {
  switch (args.option) {
    case 'install':
      install(null)
      break
    case 'open_link':
      openUrl(`https://modrinth.com/${args.item.project_type}/${args.item.slug}`)
      break
    case 'copy_link':
      navigator.clipboard.writeText(
        `https://modrinth.com/${args.item.project_type}/${args.item.slug}`,
      )
      break
  }
}
</script>

<style scoped lang="scss">
.root-container {
  display: flex;
  flex-direction: row;
  min-height: 100%;
}

.project-sidebar {
  position: fixed;
  width: calc(300px + 1.5rem);
  min-height: calc(100vh - 3.25rem);
  height: fit-content;
  max-height: calc(100vh - 3.25rem);
  padding: 1rem 0.5rem 1rem 1rem;
  overflow-y: auto;
  -ms-overflow-style: none;
  scrollbar-width: none;

  &::-webkit-scrollbar {
    width: 0;
    background: transparent;
  }
}

.sidebar-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.content-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 1rem;
  margin-left: calc(300px + 1rem);
}

.button-group {
  display: flex;
  flex-wrap: wrap;
  flex-direction: row;
  gap: 0.5rem;
}

.stats {
  display: flex;
  flex-direction: column;
  flex-wrap: wrap;
  gap: var(--gap-md);

  .stat {
    display: flex;
    flex-direction: row;
    align-items: center;
    width: fit-content;
    gap: var(--gap-xs);
    --stat-strong-size: 1.25rem;

    strong {
      font-size: var(--stat-strong-size);
    }

    p {
      margin: 0;
    }

    svg {
      min-height: var(--stat-strong-size);
      min-width: var(--stat-strong-size);
    }
  }

  .date {
    margin-top: auto;
  }
}

.tabs {
  display: flex;
  flex-direction: row;
  gap: 1rem;
  margin-bottom: var(--gap-md);
  justify-content: space-between;

  .tab {
    display: flex;
    flex-direction: row;
    align-items: center;
    border-radius: var(--border-radius);
    cursor: pointer;
    transition: background-color 0.2s ease-in-out;

    &:hover {
      background-color: var(--color-raised-bg);
    }

    &.router-view-active {
      background-color: var(--color-raised-bg);
    }
  }
}

.links {
  a {
    display: inline-flex;
    align-items: center;
    border-radius: 1rem;
    color: var(--color-text);

    svg,
    img {
      height: 1rem;
      width: 1rem;
    }

    span {
      margin-left: 0.25rem;
      text-decoration: underline;
      line-height: 2rem;
    }

    &:focus-visible,
    &:hover {
      svg,
      img,
      span {
        color: var(--color-heading);
      }
    }

    &:active {
      svg,
      img,
      span {
        color: var(--color-text-dark);
      }
    }

    &:not(:last-child)::after {
      content: '•';
      margin: 0 0.25rem;
    }
  }
}

.install-loading {
  scale: 0.2;
  height: 1rem;
  width: 1rem;
  margin-right: -1rem;

  :deep(svg) {
    color: var(--color-contrast);
  }
}

.project-sidebar-section {
  @apply p-4 flex flex-col gap-2 border-0 border-b-[1px] border-[--brand-gradient-border] border-solid;
}
</style>
