<template>
  <div class="root-container">
    <div v-if="data" class="project-sidebar">
      <Card class="sidebar-card">
        <Avatar size="lg" :src="data.icon_url"/>
        <div class="instance-info">
          <h2 class="name">{{ data.title }}</h2>
          {{ data.description }}
        </div>
        <Categories :categories="categories" :type="type" class="tags">
          <EnvironmentIndicator
              :type-only="moderation"
              :client-side="data.client_side"
              :server-side="data.server_side"
              :type="data.project_type"
          />
        </Categories>
        <hr class="card-divider">
        <div class="button-group">
          <Button color="primary" class="instance-button">
            <DownloadIcon/>
            Install
          </Button>
          <a :href="`https://modrinth.com/${data.project_type}/${data.slug}`" rel="external" target="_blank" class="btn">
            <ExternalIcon/>
            Website
          </a>
        </div>
        <hr class="card-divider">
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
            <span><span class="date-label">Created </span> {{ dayjs(data.published).fromNow() }}</span>
          </div>
          <div class="stat date">
            <UpdatedIcon aria-hidden="true" />
            <span><span class="date-label">Updated </span> {{ dayjs(data.updated).fromNow() }}</span>
          </div>
        </div>
        <hr class="card-divider">
          <div class="button-group">
            <Button class="instance-button">
              <ReportIcon/>
              Report
            </Button>
            <Button class="instance-button">
              <HeartIcon/>
              Follow
            </Button>
          </div>
        <hr class="card-divider">
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
          <a
              v-if="data.wiki_url"
              :href="data.wiki_url"
              class="title"
              rel="noopener nofollow ugc"
          >
            <WikiIcon aria-hidden="true" />
            <span>Wiki</span>
          </a>
          <a
              v-if="data.wiki_url"
              :href="data.wiki_url"
              class="title"
              rel="noopener nofollow ugc"
          >
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
            <OpenCollectiveIcon
                v-else-if="donation.id === 'open-collective'"
                aria-hidden="true"
            />
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
            }
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
            }
          ]"
        />
      </Card>
      <RouterView/>
    </div>
  </div>
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
  ExternalIcon
} from "omorphia";
import {BuyMeACoffeeIcon, DiscordIcon, PatreonIcon, PaypalIcon, KoFiIcon, OpenCollectiveIcon} from "@/assets/external";
const categories = [
  {
    name: 'magic',
    icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><path d=\'M15 4V2\'></path><path d=\'M15 16v-2\'></path><path d=\'M8 9h2\'></path><path d=\'M20 9h2\'></path><path d=\'M17.8 11.8 19 13\'></path><path d=\'M15 9h0\'></path><path d=\'M17.8 6.2 19 5\'></path><path d=\'m3 21 9-9\'></path><path d=\'M12.2 6.2 11 5\'></path></svg>',
  },
  {
    icon: '<svg viewBox=\'0 0 24 24\' fill=\'none\' stroke=\'currentColor\' stroke-width=\'2\' stroke-linecap=\'round\' stroke-linejoin=\'round\'><rect x=\'2\' y=\'7\' width=\'20\' height=\'14\' rx=\'2\' ry=\'2\'/><path d=\'M16 21V5a2 2 0 0 0-2-2h-4a2 2 0 0 0-2 2v16\'/></svg>',
    name: 'utility',
  }
]

import dayjs from 'dayjs'
import relativeTime from 'dayjs/plugin/relativeTime'
dayjs.extend(relativeTime)
</script>

<script>
export default {
  name: "Index",
  data() {
    return {
      data: null
    }
  },
  async mounted() {
    const response = await fetch('https://api.modrinth.com/v2/project/' + this.$route.params.id)
    this.data = await response.json()
  },
}
</script>

<style scoped lang="scss">
.root-container {
  display: flex;
  flex-direction: row;
  min-height: 100%;
}

.project-sidebar {
  width: 20rem;
  min-width: 20rem;
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
</style>
