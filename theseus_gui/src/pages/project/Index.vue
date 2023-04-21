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
            @click="install()"
          >
            <DownloadIcon v-if="!installed && !installing" />
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
          <Button class="instance-button">
            <ReportIcon />
            Report
          </Button>
          <Button class="instance-button">
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
            rel="noopener nofollow ugc"
          >
            <IssuesIcon aria-hidden="true" />
            <span>Issues</span>
          </a>
          <a
            v-if="data.source_url"
            :href="data.source_url"
            class="title"
            rel="noopener nofollow ugc"
          >
            <CodeIcon aria-hidden="true" />
            <span>Source</span>
          </a>
          <a v-if="data.wiki_url" :href="data.wiki_url" class="title" rel="noopener nofollow ugc">
            <WikiIcon aria-hidden="true" />
            <span>Wiki</span>
          </a>
          <a v-if="data.wiki_url" :href="data.wiki_url" class="title" rel="noopener nofollow ugc">
            <DiscordIcon aria-hidden="true" />
            <span>Discord</span>
          </a>
          <a
            v-for="(donation, index) in data.donation_urls"
            :key="index"
            :href="donation.url"
            rel="noopener nofollow ugc"
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
      />
    </div>
  </div>
  <InstallConfirmModal ref="confirmModal" />
  <InstanceInstallModal ref="modInstallModal" />
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
import { list, add_project_from_version as installMod, get as getProfile } from '@/helpers/profile'
import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
import { ofetch } from 'ofetch'
import { useRoute, useRouter } from 'vue-router'
import { ref, shallowRef, watch } from 'vue'
import InstallConfirmModal from '@/components/ui/InstallConfirmModal.vue'
import InstanceInstallModal from '@/components/ui/InstanceInstallModal.vue'
import Instance from '@/components/ui/Instance.vue'

const route = useRoute()
const router = useRouter()

const confirmModal = ref(null)
const modInstallModal = ref(null)
const loaders = ref(await get_loaders())
const categories = ref(await get_categories())
const instance = ref(null)
const installing = ref(false)

if (route.query.instance) {
  instance.value = await getProfile(route.query.instance)
}

const installed = ref(
  instance.value
    ? Object.values(instance.value.projects).some(
        (p) => p.metadata?.project?.id === route.params.id
      )
    : false
)

const [data, versions, members, dependencies] = await Promise.all([
  ofetch(`https://api.modrinth.com/v2/project/${route.params.id}`).then(shallowRef),
  ofetch(`https://api.modrinth.com/v2/project/${route.params.id}/version`).then(shallowRef),
  ofetch(`https://api.modrinth.com/v2/project/${route.params.id}/members`).then(shallowRef),
  ofetch(`https://api.modrinth.com/v2/project/${route.params.id}/dependencies`).then(shallowRef),
])

watch(
  () => route.params.id,
  () => {
    if (route.params.id) router.go()
  }
)

dayjs.extend(relativeTime)

async function install(version) {
  installing.value = true
  let queuedVersion

  if (version) {
    queuedVersion = version
  } else {
    if (data.value.project_type === 'modpack' || !route.query.instance) {
      queuedVersion = versions.value[0].id
    } else {
      queuedVersion = versions.value.find((v) =>
        v.game_versions.includes(data.value.game_versions[0])
      ).id
    }
  }

  if (data.value.project_type === 'modpack') {
    const packs = Object.values(await list())
    if (
      packs.length === 0 ||
      !packs.map((value) => value.metadata).find((pack) => pack.linked_project_id === data.value.id)
    ) {
      let id = await packInstall(queuedVersion)
      await router.push({ path: `/instance/${encodeURIComponent(id)}` })
    } else {
      confirmModal.value.show(queuedVersion)
    }
  } else {
    if (route.query.instance) {
      await installMod(route.query.instance, queuedVersion)
      installed.value = true
    } else {
      modInstallModal.value.show(data.value.id, queuedVersion)
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
  width: 19rem;
  min-width: 19rem;
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
