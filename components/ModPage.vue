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
                  : 'https://cdn.modrinth.com/placeholder.svg'
              "
              alt="mod - icon"
            />
          </div>
          <div class="info">
            <h2 class="title">{{ mod.title }}</h2>
            <p class="description">
              {{ mod.description }}
            </p>
          </div>
        </div>
        <client-only>
          <EthicalAd type="text" />
        </client-only>
        <div class="mod-navigation">
          <div class="tabs">
            <nuxt-link :to="'/mod/' + mod.id" class="tab">
              Description
            </nuxt-link>
            <nuxt-link :to="'/mod/' + mod.id + '/versions'" class="tab">
              Versions
            </nuxt-link>
            <a v-if="mod.wiki_url" :href="mod.wiki_url" class="tab">
              <ExternalIcon />
              Wiki
            </a>
            <a
              v-if="mod.issues_url"
              :href="mod.issues_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              Issues
            </a>
            <a
              v-if="mod.source_url"
              :href="mod.source_url"
              target="_blank"
              class="tab"
            >
              <ExternalIcon />
              Source
            </a>
            <nuxt-link
              v-if="
                this.$auth.loggedIn &&
                members.find((x) => x.user_id === this.$auth.user.id)
              "
              :to="'/mod/' + mod.id + '/settings'"
              class="tab"
            >
              Settings
            </nuxt-link>
            <div class="filler" />
          </div>
        </div>
        <div class="mod-content">
          <slot />
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
                  versions[versions.length - 1]
                    ? versions[versions.length - 1].game_versions[0]
                      ? versions[versions.length - 1].game_versions[0]
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
        <div v-if="versions.length > 0" class="section">
          <h3>Featured Versions</h3>
          <div
            v-for="version in versions"
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
                  <nuxt-link :to="'/mod/' + mod.id + '/version/' + version.id">
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
                  · {{ version.game_versions[0] }}
                </span>
              </div>
            </div>
          </div>
        </div>
        <m-footer class="footer" />
      </section>
    </div>
  </div>
</template>

<script>
import EthicalAd from '@/components/EthicalAd'

import Categories from '@/components/Categories'
import MFooter from '@/components/MFooter'

import axios from 'axios'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import TagIcon from '~/assets/images/utils/tag.svg?inline'

import ExternalIcon from '~/assets/images/utils/external.svg?inline'

import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  name: 'ModPage',
  components: {
    MFooter,
    Categories,
    EthicalAd,
    ExternalIcon,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
    CalendarIcon,
    EditIcon,
    TagIcon,
  },
  props: {
    mod: {
      type: Object,
      default() {
        return {}
      },
    },
    versions: {
      type: Array,
      default() {
        return []
      },
    },
    members: {
      type: Array,
      default() {
        return []
      },
    },
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
  },
}
</script>

<style lang="scss" scoped>
.header {
  @extend %row;
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
      height: 100%;
      color: var(--color-text-dark);
    }
  }
}

.mod-navigation {
  @extend %card-spaced-b;
  padding-bottom: 0.2rem;
}

.mod-info {
  width: 30rem;
  height: auto;
  margin-left: var(--spacing-card-lg);

  .section {
    padding: var(--spacing-card-sm);
    @extend %card-spaced-b;
    margin-top: var(--spacing-card-lg);
  }

  h3 {
    @extend %large-label;
  }

  .mod-stats {
    display: flex;
    flex-wrap: wrap;
    margin-top: 0;
    margin-left: 5px;
    p {
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

@media screen and (max-width: 1400px) {
  .mod-info {
    display: none;
  }
}
</style>
