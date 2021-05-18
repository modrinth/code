<template>
  <div class="page-container">
    <div class="page-contents">
      <div class="content">
        <div class="header">
          <div class="icon">
            <img
              :src="
                mod.icon_url
                  ? mod.icon_url
                  : 'https://cdn.modrinth.com/placeholder.svg?inline'
              "
              alt="mod - icon"
            />
          </div>
          <div class="info">
            <h1 class="title">{{ mod.title }}</h1>
            <p class="description">
              {{ mod.description }}
            </p>
            <div class="alt-nav">
              <p>
                <nuxt-link to="/mods"> Mods </nuxt-link>
                >
                <nuxt-link :to="'/mod/' + (mod.slug ? mod.slug : mod.id)">{{
                  mod.title
                }}</nuxt-link>
                <span v-if="linkBar.length > 0"> ></span>
                <nuxt-link
                  v-for="(link, index) in linkBar"
                  :key="index"
                  :to="/mod/ + (mod.slug ? mod.slug : mod.id) + '/' + link[1]"
                  >{{ link[0] }}
                  <span v-if="index !== linkBar.length - 1"> > </span>
                </nuxt-link>
              </p>
            </div>
          </div>
          <div class="buttons">
            <nuxt-link
              v-if="this.$auth.user"
              :to="`/report/create?id=${mod.id}&t=mod`"
              class="iconified-button"
            >
              <ReportIcon />
              Report
            </nuxt-link>
            <button
              v-if="userFollows && !userFollows.includes(mod.id)"
              class="iconified-button"
              @click="followMod"
            >
              <FollowIcon />
              Follow
            </button>
            <button
              v-if="userFollows && userFollows.includes(mod.id)"
              class="iconified-button"
              @click="unfollowMod"
            >
              <FollowIcon fill="currentColor" />
              Unfollow
            </button>
          </div>
        </div>
        <Advertisement
          v-if="mod.status === 'approved' || mod.status === 'unlisted'"
          type="banner"
          small-screen="square"
          ethical-ads-small
          ethical-ads-big
        />
        <div class="mod-navigation">
          <div class="tabs">
            <nuxt-link
              :to="'/mod/' + (mod.slug ? mod.slug : mod.id)"
              class="tab"
            >
              <span>Description</span>
            </nuxt-link>
            <nuxt-link
              :to="'/mod/' + (mod.slug ? mod.slug : mod.id) + '/versions'"
              class="tab"
            >
              <span>Versions</span>
            </nuxt-link>
            <nuxt-link
              v-if="currentMember"
              :to="'/mod/' + (mod.slug ? mod.slug : mod.id) + '/settings'"
              class="tab"
            >
              <span>Settings</span>
            </nuxt-link>
            <a
              v-if="mod.wiki_url"
              :href="mod.wiki_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              <span>Wiki</span>
            </a>
            <a
              v-if="mod.issues_url"
              :href="mod.issues_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              <span>Issues</span>
            </a>
            <a
              v-if="mod.source_url"
              :href="mod.source_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              <span>Source</span>
            </a>
            <a
              v-if="mod.discord_url"
              :href="mod.discord_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              <span>Discord</span>
            </a>
            <div class="filler" />
          </div>
        </div>
        <div class="mod-content">
          <NuxtChild
            :mod="mod"
            :versions="versions"
            :featured-versions="featuredVersions"
            :members="members"
            :current-member="currentMember"
            :link-bar.sync="linkBar"
          />
        </div>
      </div>
      <section class="mod-info">
        <div class="mod-stats section">
          <div class="stat">
            <DownloadIcon />
            <div class="info">
              <h4>Downloads</h4>
              <p class="value">{{ formatNumber(mod.downloads) }}</p>
            </div>
          </div>
          <div class="stat">
            <CalendarIcon />
            <div class="info">
              <h4>Created</h4>
              <p
                v-tooltip="
                  $dayjs(mod.published).format(
                    '[Created on] YYYY-MM-DD [at] HH:mm A'
                  )
                "
                class="value"
              >
                {{ $dayjs(mod.published).fromNow() }}
              </p>
            </div>
          </div>
          <div class="stat">
            <TagIcon />
            <div class="info">
              <h4>Available For</h4>
              <p class="value">
                {{
                  versions[0]
                    ? versions[0].game_versions[0]
                      ? versions[0].game_versions[
                          versions[0].game_versions.length - 1
                        ]
                      : 'None'
                    : 'None'
                }}
              </p>
            </div>
          </div>
          <div class="stat">
            <EditIcon />
            <div class="info">
              <h4>Updated</h4>
              <p
                v-tooltip="
                  $dayjs(mod.updated).format(
                    '[Updated on] YYYY-MM-DD [at] HH:mm A'
                  )
                "
                class="value"
              >
                {{ $dayjs(mod.updated).fromNow() }}
              </p>
            </div>
          </div>
          <div class="stat">
            <ClientIcon />
            <div class="info">
              <h4>Client Side</h4>
              <p class="value capitalize">{{ mod.client_side }}</p>
            </div>
          </div>
          <div class="stat">
            <ServerIcon />
            <div class="info">
              <h4>Server Side</h4>
              <p class="value capitalize">{{ mod.server_side }}</p>
            </div>
          </div>
          <div class="stat">
            <FileTextIcon />
            <div class="info">
              <h4>License</h4>
              <p v-tooltip="mod.license.name" class="value ellipsis">
                <a
                  v-if="mod.license.url ? mod.license.url : '#'"
                  :href="mod.license.url"
                >
                  {{ mod.license.id.toUpperCase() }}</a
                >
              </p>
            </div>
          </div>
          <div class="stat">
            <CodeIcon />
            <div class="info">
              <h4>Project ID</h4>
              <p class="value">{{ mod.id }}</p>
            </div>
          </div>
          <Categories :categories="mod.categories.concat(mod.loaders)" />
        </div>
        <div class="section">
          <h3>Members</h3>
          <div
            v-for="member in members"
            :key="member.user_id"
            class="team-member columns"
          >
            <img :src="member.avatar_url" alt="profile-picture" />
            <div class="member-info">
              <nuxt-link :to="'/user/' + member.user_id">
                <h4>{{ member.name }}</h4>
              </nuxt-link>
              <h3>{{ member.role }}</h3>
            </div>
          </div>
        </div>
        <div v-if="featuredVersions.length > 0" class="section">
          <h3>Featured Versions</h3>
          <div
            v-for="version in featuredVersions"
            :key="version.id"
            class="featured-version"
          >
            <a
              :href="findPrimary(version).url"
              class="download"
              @click.prevent="
                downloadFile(
                  findPrimary(version).hashes.sha1,
                  findPrimary(version).url
                )
              "
            >
              <DownloadIcon />
            </a>
            <div class="info">
              <div class="top">
                <span
                  v-if="version.version_type === 'release'"
                  class="badge green"
                >
                  Release
                </span>
                <span
                  v-if="version.version_type === 'beta'"
                  class="badge yellow"
                >
                  Beta
                </span>
                <span v-if="version.version_type === 'alpha'" class="badge red">
                  Alpha
                </span>
                <h4 class="title">
                  <nuxt-link
                    :to="
                      '/mod/' +
                      (mod.slug ? mod.slug : mod.id) +
                      '/version/' +
                      version.id
                    "
                  >
                    {{ version.name }}
                  </nuxt-link>
                </h4>
              </div>
              <div class="bottom">
                <span class="version-number limit-text-width">
                  {{ version.version_number }} ·
                </span>
                <FabricIcon
                  v-if="version.loaders.includes('fabric')"
                  class="loader"
                />
                <ForgeIcon
                  v-if="version.loaders.includes('forge')"
                  class="loader"
                />
                <span
                  v-if="version.game_versions.length > 0"
                  class="game-version limit-text-width"
                >
                  ·
                  {{ version.game_versions[version.game_versions.length - 1] }}
                </span>
              </div>
            </div>
          </div>
        </div>
        <div
          v-if="mod.donation_urls && mod.donation_urls.length > 0"
          class="section"
        >
          <h3>Donation Links</h3>
          <div
            v-for="(item, index) in mod.donation_urls"
            :key="index"
            class="links"
          >
            <a :href="item.url" class="link">
              <ExternalIcon />
              {{ item.platform }}
            </a>
          </div>
        </div>
        <Advertisement
          v-if="mod.status === 'approved' || mod.status === 'unlisted'"
          type="square"
          small-screen="destroy"
        />
        <m-footer class="footer" />
      </section>
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import Categories from '~/components/ui/search/Categories'
import MFooter from '~/components/layout/MFooter'

import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import TagIcon from '~/assets/images/utils/tag.svg?inline'
import ClientIcon from '~/assets/images/utils/client.svg?inline'
import ServerIcon from '~/assets/images/utils/server.svg?inline'
import FileTextIcon from '~/assets/images/utils/file-text.svg?inline'
import CodeIcon from '~/assets/images/sidebar/mod.svg?inline'
import ReportIcon from '~/assets/images/utils/report.svg?inline'
import FollowIcon from '~/assets/images/utils/heart.svg?inline'

import ExternalIcon from '~/assets/images/utils/external.svg?inline'

import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'
import Advertisement from '~/components/ads/Advertisement'

export default {
  name: 'ModPage',
  components: {
    Advertisement,
    MFooter,
    Categories,
    ExternalIcon,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
    CalendarIcon,
    EditIcon,
    TagIcon,
    ClientIcon,
    ServerIcon,
    FileTextIcon,
    CodeIcon,
    ReportIcon,
    FollowIcon,
  },
  async asyncData(data) {
    try {
      const mod = (
        await axios.get(
          `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
          data.$auth.headers
        )
      ).data

      const [members, versions, featuredVersions, userFollows] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
          axios.get(`https://api.modrinth.com/api/v1/mod/${mod.id}/version`),
          axios.get(
            `https://api.modrinth.com/api/v1/mod/${mod.id}/version?featured=true`
          ),
          axios.get(
            data.$auth.user
              ? `https://api.modrinth.com/api/v1/user/${data.$auth.user.id}/follows`
              : `https://api.modrinth.com`,
            data.$auth.headers
          ),
        ])
      ).map((it) => it.data)

      const users = (
        await axios.get(
          `https://api.modrinth.com/api/v1/users?ids=${JSON.stringify(
            members.map((it) => it.user_id)
          )}`,
          data.$auth.headers
        )
      ).data

      users.forEach((it) => {
        const index = members.findIndex((x) => x.user_id === it.id)
        members[index].avatar_url = it.avatar_url
        members[index].name = it.username
      })

      const currentMember = data.$auth.user
        ? members.find((x) => x.user_id === data.$auth.user.id)
        : null

      return {
        mod,
        versions,
        featuredVersions,
        members,
        currentMember,
        userFollows: userFollows.name ? null : userFollows,
        linkBar: [],
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Mod not found',
      })
    }
  },
  methods: {
    formatNumber(x) {
      return x.toString().replace(/\B(?=(\d{3})+(?!\d))/g, ',')
    },
    findPrimary(version) {
      let file = version.files.find((x) => x.primary)

      if (!file) {
        file = version.files[0]
      }

      if (!file) {
        file = { url: `/mod/${this.mod.id}/version/${version.id}` }
      }

      return file
    },
    async downloadFile(hash, url) {
      await axios.get(
        `https://api.modrinth.com/api/v1/version_file/${hash}/download`
      )

      const elem = document.createElement('a')
      elem.download = hash
      elem.href = url
      elem.click()
    },
    async followMod() {
      await axios.post(
        `https://api.modrinth.com/api/v1/mod/${this.mod.id}/follow`,
        {},
        this.$auth.headers
      )

      this.userFollows.push(this.mod.id)
    },
    async unfollowMod() {
      await axios.delete(
        `https://api.modrinth.com/api/v1/mod/${this.mod.id}/follow`,
        this.$auth.headers
      )

      this.userFollows.splice(this.userFollows.indexOf(this.mod.id), 1)
    },
  },
  head() {
    return {
      title: this.mod.title + ' - Modrinth',
      meta: [
        {
          hid: 'og:type',
          name: 'og:type',
          content: 'website',
        },
        {
          hid: 'og:title',
          name: 'og:title',
          content: this.mod.title,
        },
        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.mod.title,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.mod.description,
        },
        {
          hid: 'description',
          name: 'description',
          content:
            this.mod.description +
            ' View other minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform supporting both the Forge and Fabric mod loaders.',
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/mod/${this.mod.id}`,
        },
        {
          hid: 'og:image',
          name: 'og:image',
          content: this.mod.icon_url
            ? this.mod.icon_url
            : 'https://cdn.modrinth.com/placeholder.png',
        },
      ],
    }
  },
}
</script>

<style lang="scss" scoped>
.header {
  display: flex;
  flex-direction: column;
  align-items: center;
  text-align: center;

  @extend %card-spaced-b;
  width: 100%;
  .icon {
    margin: auto 0;
    img {
      width: 6rem;
      height: 6rem;
      margin: var(--spacing-card-md);
      border-radius: var(--size-rounded-icon);
      object-fit: contain;
    }
  }
  .info {
    @extend %column;
    .title {
      margin: var(--spacing-card-md) var(--spacing-card-md) 0 0;
      color: var(--color-text-dark);
      font-size: var(--font-size-lg);
    }
    .description {
      margin: var(--spacing-card-sm) var(--spacing-card-md) 0 0;
      color: var(--color-text-dark);
    }
    .alt-nav {
      margin: var(--spacing-card-sm) var(--spacing-card-md) 0 0;
      p {
        margin: 0;
      }
    }
  }
  .buttons {
    @extend %row;
    margin: var(--spacing-card-md) var(--spacing-card-md) var(--spacing-card-md)
      0;

    button,
    a {
      margin: 0.2rem 0 0 0.2rem;
    }
  }

  @media screen and (min-width: 1024px) {
    align-items: unset;
    text-align: unset;
    flex-direction: row;

    .buttons {
      flex-direction: column;
      margin-left: auto;
    }
  }
}

.mod-navigation {
  @extend %card-spaced-b;
  padding-bottom: 0.2rem;
}

.mod-info {
  height: auto;

  @media screen and (min-width: 1024px) {
    width: 30rem;
    margin-left: var(--spacing-card-lg);
  }

  .section {
    padding: var(--spacing-card-sm);
    @extend %card-spaced-b;
  }

  h3 {
    @extend %large-label;
  }

  .mod-stats {
    display: flex;
    flex-wrap: wrap;
    margin-top: 0;
    p {
      max-width: 6rem;
      overflow: hidden;
      white-space: nowrap;
      text-overflow: '';
      margin: 3px;
    }
    .stat {
      width: 8.5rem;
      margin: 0.75rem;
      @extend %stat;

      svg {
        padding: 0.25rem;
        border-radius: 50%;
        background-color: var(--color-button-bg);
      }
    }
  }

  .team-member {
    margin-left: 5px;
    margin-bottom: 10px;

    img {
      border-radius: var(--size-rounded-icon);
      height: 50px;
      width: 50px;
    }
    .member-info {
      max-width: 150px;
      overflow: hidden;
      margin: auto 0 auto 0.5rem;
      h4 {
        font-weight: normal;
        margin: 0;
      }
      h3 {
        margin-top: 0.1rem;
        margin-bottom: 0;
      }
    }
  }

  .featured-version {
    @extend %row;
    padding-top: var(--spacing-card-sm);
    padding-bottom: var(--spacing-card-sm);
    .download {
      display: flex;
      align-items: center;
      height: 2.25rem;
      width: 2.25rem;
      border-radius: 2rem;
      background-color: var(--color-button-bg);
      margin-right: var(--spacing-card-sm);
      svg {
        width: 1.25rem;
        margin: auto;
      }
      flex-shrink: 0;
    }
    .info {
      @extend %column;
      font-size: var(--font-size-xs);
      .top {
        @extend %row;
        .badge {
          font-size: var(--font-size-xs);
          margin-right: var(--spacing-card-sm);
        }
        .title {
          margin: auto 0;
        }
      }
      .bottom {
        margin-top: 0.25rem;
        @extend %row;
        .loader {
          height: 1rem;
        }
      }
    }
  }

  .links {
    padding: 0.5rem 1rem;

    .link {
      display: flex;
      align-items: center;
      padding: 0.25rem 0;

      svg {
        width: 1rem;
        height: 1rem;
        margin-right: 0.3rem;
      }

      &:hover,
      &:focus {
        padding-bottom: calc(0.25rem - 3px);
        border-bottom: 3px solid var(--color-brand-disabled);
        color: var(--color-text-medium);
      }
    }
  }
}

.limit-text-width {
  display: inline-block;
  height: 1em;
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

@media screen and (max-width: 550px) {
  .title a {
    display: none;
  }
}

@media screen and (max-width: 800px) {
  .mod-navigation {
    display: block;
    overflow-x: auto;
    overflow-wrap: break-word;
    overflow-y: hidden;
  }
}
/*
@media screen and (max-width: 1400px) {
  .mod-info {
    display: none;
  }
}
*/
</style>
