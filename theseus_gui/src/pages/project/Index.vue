<template>
  <div class="root-container">
    <div v-if="data" class="project-sidebar">
      <Instance v-if="instance" :instance="instance" small />
      <Card class="sidebar-card">
        <Avatar size="lg" :src="data.icon_url" />
        <div class="instance-info">
          <h2 class="name">{{ data.title }}</h2>
          {{ data.description }}
        </div>
        <Categories
          class="tags"
          type=""
          :categories="[
            ...categories.filter(
              (cat) => data.categories.includes(cat.name) && cat.project_type === 'mod'
            ),
            ...loaders.filter(
              (loader) =>
                data.categories.includes(loader.name) &&
                loader.supported_project_types?.includes('modpack')
            ),
          ]"
        >
          <EnvironmentIndicator
            :client-side="data.client_side"
            :server-side="data.server_side"
            :type="data.project_type"
          />
        </Categories>
        <hr class="card-divider" />
        <div class="button-group">
          <Button
            color="primary"
            class="instance-button"
            :disabled="installed === true || installing === true"
            @click="install(null)"
          >
            <DownloadIcon v-if="!installed && !installing" />
            <CheckIcon v-else-if="installed" />
            {{ installing ? 'Installing...' : installed ? 'Installed' : 'Install' }}
          </Button>
          <a
            :href="`https://modrinth.com/${data.project_type}/${data.slug}`"
            rel="external"
            target="_blank"
            class="btn"
          >
            <ExternalIcon />
            Site
          </a>
        </div>
        <hr class="card-divider" />
        <div class="stats">
          <div class="stat">
            <DownloadIcon aria-hidden="true" />
            <p>
              <strong>{{ formatNumber(data.downloads) }}</strong>
              <span class="stat-label"> download<span v-if="data.downloads !== '1'">s</span></span>
            </p>
          </div>
          <div class="stat">
            <HeartIcon aria-hidden="true" />
            <p>
              <strong>{{ formatNumber(data.followers) }}</strong>
              <span class="stat-label"> follower<span v-if="data.followers !== '1'">s</span></span>
            </p>
          </div>
          <div class="stat date">
            <CalendarIcon aria-hidden="true" />
            <span
              ><span class="date-label">Created </span> {{ dayjs(data.published).fromNow() }}</span
            >
          </div>
          <div class="stat date">
            <UpdatedIcon aria-hidden="true" />
            <span
              ><span class="date-label">Updated </span> {{ dayjs(data.updated).fromNow() }}</span
            >
          </div>
        </div>
        <hr class="card-divider" />
        <div class="button-group">
          <Button class="instance-button" disabled>
            <ReportIcon />
            Report
          </Button>
          <Button class="instance-button" disabled>
            <HeartIcon />
            Follow
          </Button>
        </div>
        <hr class="card-divider" />
        <div class="links">
          <a
            v-if="data.issues_url"
            :href="data.issues_url"
            class="title"
            rel="noopener nofollow ugc external"
            target="_blank"
          >
            <IssuesIcon aria-hidden="true" />
            <span>Issues</span>
          </a>
          <a
            v-if="data.source_url"
            :href="data.source_url"
            class="title"
            target="_blank"
            rel="noopener nofollow ugc external"
          >
            <CodeIcon aria-hidden="true" />
            <span>Source</span>
          </a>
          <a
            v-if="data.wiki_url"
            :href="data.wiki_url"
            class="title"
            rel="noopener nofollow ugc external"
            target="_blank"
          >
            <WikiIcon aria-hidden="true" />
            <span>Wiki</span>
          </a>
          <a
            v-if="data.wiki_url"
            :href="data.wiki_url"
            class="title"
            rel="noopener nofollow ugc external"
            target="_blank"
          >
            <DiscordIcon aria-hidden="true" />
            <span>Discord</span>
          </a>
          <a
            v-for="(donation, index) in data.donation_urls"
            :key="index"
            :href="donation.url"
            target="_blank"
            rel="noopener nofollow ugc external"
          >
            <BuyMeACoffeeIcon v-if="donation.id === 'bmac'" aria-hidden="true" />
            <PatreonIcon v-else-if="donation.id === 'patreon'" aria-hidden="true" />
            <KoFiIcon v-else-if="donation.id === 'ko-fi'" aria-hidden="true" />
            <PaypalIcon v-else-if="donation.id === 'paypal'" aria-hidden="true" />
            <OpenCollectiveIcon v-else-if="donation.id === 'open-collective'" aria-hidden="true" />
            <HeartIcon v-else-if="donation.id === 'github'" />
            <CoinsIcon v-else />
            <span v-if="donation.id === 'bmac'">Buy Me a Coffee</span>
            <span v-else-if="donation.id === 'patreon'">Patreon</span>
            <span v-else-if="donation.id === 'paypal'">PayPal</span>
            <span v-else-if="donation.id === 'ko-fi'">Ko-fi</span>
            <span v-else-if="donation.id === 'github'">GitHub Sponsors</span>
            <span v-else>Donate</span>
          </a>
        </div>
      </Card>
    </div>
    <div v-if="data" class="content-container">
      <Promotion />
      <Card class="tabs">
        <NavRow
          v-if="data.gallery.length > 0"
          :links="[
            {
              label: 'Description',
              href: `/project/${$route.params.id}/`,
            },
            {
              label: 'Versions',
              href: `/project/${$route.params.id}/versions`,
            },
            {
              label: 'Gallery',
              href: `/project/${$route.params.id}/gallery`,
            },
          ]"
        />
        <NavRow
          v-else
          :links="[
            {
              label: 'Description',
              href: `/project/${$route.params.id}/`,
            },
            {
              label: 'Versions',
              href: `/project/${$route.params.id}/versions`,
            },
          ]"
        />
      </Card>
      <RouterView
        :project="data"
        :versions="versions"
        :members="members"
        :dependencies="dependencies"
        :install="install"
        :installed="installed"
      />
    </div>
  </div>
  <InstallConfirmModal ref="confirmModal" />
  <InstanceInstallModal ref="modInstallModal" />
  <IncompatibilityWarningModal ref="incompatibilityWarning" />
</template>

<script setup>
import {
  Card,
  Avatar,
  Button,
  DownloadIcon,
  ReportIcon,
  HeartIcon,
  Categories,
  EnvironmentIndicator,
  UpdatedIcon,
  CalendarIcon,
  IssuesIcon,
  WikiIcon,
  Promotion,
  NavRow,
  CoinsIcon,
  CodeIcon,
  formatNumber,
  ExternalIcon,
  CheckIcon,
} from 'omorphia'
import {
  BuyMeACoffeeIcon,
  DiscordIcon,
  PatreonIcon,
  PaypalIcon,
  KoFiIcon,
  OpenCollectiveIcon,
} from '@/assets/external'
import { get_categories, get_loaders } from '@/helpers/tags'
import { install as packInstall } from '@/helpers/pack'
import { list, add_project_from_version as installMod, check_installed } from '@/helpers/profile'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { useRoute, useRouter } from 'vue-router'
import { ref, shallowRef, watch } from 'vue'
import { installVersionDependencies } from '@/helpers/utils'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import InstanceInstallModal from '@/components/ui/InstanceInstallModal.vue'
import Instance from '@/components/ui/Instance.vue'
import { useSearch } from '@/store/search'
import { useBreadcrumbs } from '@/store/breadcrumbs'
import IncompatibilityWarningModal from '@/components/ui/IncompatibilityWarningModal.vue'
import { useFetch } from '@/helpers/fetch.js'
import { handleError } from '@/store/notifications.js'

const searchStore = useSearch()

const route = useRoute()
const router = useRouter()
const breadcrumbs = useBreadcrumbs()

const confirmModal = ref(null)
const modInstallModal = ref(null)
const incompatibilityWarning = ref(null)
const instance = ref(searchStore.instanceContext)
const installing = ref(false)

const [data, versions, members, dependencies, categories, loaders] = await Promise.all([
  useFetch(`https://api.modrinth.com/v2/project/${route.params.id}`, 'project').then(shallowRef),
  useFetch(`https://api.modrinth.com/v2/project/${route.params.id}/version`, 'project').then(
    shallowRef
  ),
  useFetch(`https://api.modrinth.com/v2/project/${route.params.id}/members`, 'project').then(
    shallowRef
  ),
  useFetch(`https://api.modrinth.com/v2/project/${route.params.id}/dependencies`, 'project').then(
    shallowRef
  ),
  get_loaders().then(ref).catch(handleError),
  get_categories().then(ref).catch(handleError),
])

const installed = ref(
  instance.value && (await check_installed(instance.value.path, data.value.id).catch(handleError))
)

breadcrumbs.setName('Project', data.value.title)

watch(
  () => route.params.id,
  () => {
    if (route.params.id) router.go()
  }
)

dayjs.extend(relativeTime)

const markInstalled = () => {
  installed.value = true
}

async function install(version) {
  installing.value = true
  let queuedVersionData

  if (version) {
    queuedVersionData = versions.value.find((v) => v.id === version)
  } else {
    if (data.value.project_type === 'modpack' || !instance.value) {
      queuedVersionData = versions.value[0]
    } else {
      queuedVersionData = versions.value.find((v) =>
        v.game_versions.includes(data.value.game_versions[0])
      )
    }
  }

  if (data.value.project_type === 'modpack') {
    const packs = Object.values(await list(true).catch(handleError))
    if (
      packs.length === 0 ||
      !packs
        .map((value) => value.metadata)
        .find((pack) => pack.linked_data?.project_id === data.value.id)
    ) {
      await packInstall(queuedVersionData.id, data.value.title, data.value.icon_url).catch(
        handleError
      )
    } else {
      confirmModal.value.show(queuedVersionData.id, data.value.title, data.value.icon_url)
    }
  } else {
    if (instance.value) {
      if (!version) {
        const gameVersion = instance.value.metadata.game_version
        const loader = instance.value.metadata.loader
        const selectedVersion = versions.value.find(
          (v) =>
            v.game_versions.includes(gameVersion) &&
            (data.value.project_type === 'mod'
              ? v.loaders.includes(loader) || v.loaders.includes('minecraft')
              : true)
        )
        if (!selectedVersion) {
          incompatibilityWarning.value.show(
            instance.value,
            data.value.title,
            versions.value,
            markInstalled
          )
          installing.value = false
          return
        } else {
          queuedVersionData = selectedVersion
          await installMod(instance.value.path, selectedVersion.id).catch(handleError)
          installVersionDependencies(instance.value, queuedVersionData)
        }
      } else {
        const gameVersion = instance.value.metadata.game_version
        const loader = instance.value.metadata.loader
        const compatible = versions.value.some(
          (v) =>
            v.game_versions.includes(gameVersion) &&
            (data.value.project_type === 'mod'
              ? v.loaders.includes(loader) || v.loaders.includes('minecraft')
              : true)
        )
        if (compatible) {
          await installMod(instance.value.path, queuedVersionData.id).catch(handleError)
          await installVersionDependencies(instance.value, queuedVersionData)
        } else {
          incompatibilityWarning.value.show(
            instance.value,
            data.value.title,
            [queuedVersionData],
            markInstalled
          )
          installing.value = false
          return
        }
      }
      installed.value = true
    } else {
      if (version) {
        modInstallModal.value.show(data.value.id, [
          versions.value.find((v) => v.id === queuedVersionData.id),
        ])
      } else {
        modInstallModal.value.show(data.value.id, versions.value)
      }
    }
  }

  installing.value = false
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
  width: 20rem;
  min-height: calc(100vh - 3.25rem);
  height: fit-content;
  max-height: calc(100vh - 3.25rem);
  overflow-y: auto;
  background: var(--color-raised-bg);
  padding: 1rem;
}

.sidebar-card {
  display: flex;
  flex-direction: column;
  gap: 1rem;
  background-color: var(--color-bg);
}

.content-container {
  display: flex;
  flex-direction: column;
  width: 100%;
  padding: 1rem;
  margin-left: 20rem;
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
</style>
