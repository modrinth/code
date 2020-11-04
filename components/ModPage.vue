<template>
  <div class="columns">
    <div class="content column-grow-5">
      <div class="mod-header columns">
        <img
          :src="
            mod.icon_url
              ? mod.icon_url
              : 'https://cdn.modrinth.com/placeholder.svg'
          "
          alt="mod-icon"
        />
        <div class="mod-header-text">
          <div class="columns title">
            <h2>{{ mod.title }}</h2>
            <nuxt-link
              :to="'/user/' + members.find((x) => x.role === 'Owner').user_id"
            >
              <p>by {{ members.find((x) => x.role === 'Owner').name }}</p>
            </nuxt-link>
          </div>
          <p>{{ mod.description }}</p>
        </div>
      </div>
      <div class="mod-navigation">
        <nuxt-link :to="'/mod/' + mod.id">
          <InfoIcon />
          Description
        </nuxt-link>
        <nuxt-link :to="'/mod/' + mod.id + '/versions'">
          <VersionIcon />
          Versions
        </nuxt-link>
        <nuxt-link
          v-if="
            this.$auth.loggedIn &&
            members.find((x) => x.user_id === this.$auth.user.id)
          "
          :to="'/mod/' + mod.id + '/settings'"
        >
          <SettingsIcon />
          Settings
        </nuxt-link>
        <a v-if="mod.wiki_url" :href="mod.wiki_url">
          <ExternalIcon />
          Wiki
        </a>
        <a v-if="mod.issues_url" :href="mod.issues_url">
          <ExternalIcon />
          Issues
        </a>
        <a v-if="mod.source_url" :href="mod.source_url">
          <ExternalIcon />
          Source Code
        </a>
        <div class="filler" />
      </div>
      <slot />
    </div>
    <div>
      <section class="mod-info">
        <div class="mod-stats">
          <h3>Info</h3>
          <p>{{ mod.downloads }} Downloads</p>
          <p>Created {{ $dayjs(mod.published).fromNow() }}</p>
          <p>Updated {{ $dayjs(mod.updated).fromNow() }}</p>
        </div>
        <div>
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
              <p>{{ member.role }}</p>
            </div>
          </div>
        </div>
        <div v-if="versions.length > 0">
          <h3>Featured Versions</h3>
          <div
            v-for="version in versions"
            :key="version.id"
            class="featured-version columns"
          >
            <div class="version-info">
              <div class="columns">
                <h4 class="limit-text-width">
                  {{ version.name }}
                </h4>
                <p
                  v-if="version.version_type === 'release'"
                  class="badge green"
                >
                  Release
                </p>
                <p v-if="version.version_type === 'beta'" class="badge yellow">
                  Beta
                </p>
                <p v-if="version.version_type === 'alpha'" class="badge red">
                  Alpha
                </p>
              </div>
              <div class="columns info-2">
                <p class="version-number limit-text-width">
                  {{ version.version_number }}
                </p>
                <FabricIcon
                  v-if="version.loaders.includes('fabric')"
                  stroke="#AC6C3A"
                />
                <ForgeIcon
                  v-if="version.loaders.includes('forge')"
                  stroke="#8B81E6"
                />
                <p
                  v-if="version.game_versions.length > 0"
                  class="game-version limit-text-width"
                >
                  {{ version.game_versions[0] }}
                </p>
              </div>
            </div>
            <nuxt-link :to="'/mod/' + mod.id + '/version/' + version.id">
              <DownloadIcon />
            </nuxt-link>
          </div>
          <EthicalAd type="image" />
        </div>
      </section>
    </div>
  </div>
</template>

<script>
import EthicalAd from '@/components/EthicalAd'

import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import ExternalIcon from '~/assets/images/utils/external.svg?inline'
import InfoIcon from '~/assets/images/utils/info.svg?inline'
import VersionIcon from '~/assets/images/utils/version.svg?inline'
import SettingsIcon from '~/assets/images/utils/settings.svg?inline'

import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  name: 'ModPage',
  components: {
    EthicalAd,
    ExternalIcon,
    InfoIcon,
    VersionIcon,
    SettingsIcon,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
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
}
</script>

<style lang="scss">
.mod-header {
  align-items: center;

  img {
    border-radius: var(--size-rounded-md);
    width: 150px;
    height: 150px;
    object-fit: cover;
  }

  .mod-header-text {
    margin-left: 15px;

    .title {
      align-items: end;

      h2 {
        margin: 0;
      }

      p {
        align-self: flex-end;
        margin: 0;
        padding: 0 0 2px 5px;
      }
    }
  }
}

.mod-navigation {
  display: flex;
  margin-top: 20px;
  overflow-y: auto;

  a {
    user-select: none;
    display: flex;
    align-items: center;
    padding: 10px 20px;
    border-bottom: 2px solid var(--color-grey-2);

    svg {
      margin-right: 10px;
    }

    &:hover,
    &:focus {
      border-bottom: 2px solid var(--color-grey-3);
    }

    &.nuxt-link-exact-active {
      border-bottom: 2px solid var(--color-brand);
    }
  }

  .filler {
    flex-grow: 1;
    border-bottom: 2px solid var(--color-grey-2);
  }
}

.mod-info {
  top: 1rem;
  position: sticky;
  min-width: 270px;
  max-width: 270px;
  margin: 1rem;
  padding: 0 0.75rem 0 1rem;
  overflow-y: auto;
  background-color: var(--color-bg);
  border: 1px solid var(--color-grey-2);
  border-radius: var(--size-rounded-sm);

  h3 {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin: 1.5rem 0 0.5rem 0;
    text-transform: uppercase;
  }

  .mod-stats {
    margin-left: 5px;
    p {
      color: var(--color-grey-4);
      margin: 3px;
    }
  }

  .team-member {
    margin-left: 5px;
    margin-bottom: 10px;
    border: 1px solid var(--color-grey-1);
    border-radius: var(--size-rounded-sm);

    img {
      border-radius: var(--size-rounded-sm);
      border-top-right-radius: 0;
      border-bottom-right-radius: 0;
      height: 50px;
      width: 50px;
    }
    .member-info {
      max-width: 150px;
      overflow: hidden;
      margin: auto 0 auto 20px;
      h4 {
        font-weight: normal;
        margin: 0;
      }
      p {
        color: var(--color-grey-4);
        font-weight: lighter;
        font-size: 12pt;
        margin: 0;
      }
    }
  }

  .featured-version {
    margin-left: 5px;
    margin-bottom: 10px;
    border: 1px solid var(--color-grey-1);
    border-radius: var(--size-rounded-sm);

    .version-info {
      padding: 5px 10px;
      h4 {
        max-width: 120px;
        font-weight: normal;
        margin: 0 10px 0 0;
      }
      .badge {
        margin: 0;
        display: inline-block;
      }
      .info-2 {
        overflow: hidden;
        max-width: 180px;
        align-items: center;

        .version-number {
          max-width: 80px;
        }

        .game-version {
          max-width: 120px;
        }

        p {
          color: var(--color-grey-4);
          font-weight: lighter;
          margin: 0 10px 0 0;
        }

        svg {
          min-width: 24px;
          min-height: 24px;
          margin-right: 10px;
        }
      }
    }

    a {
      display: table-cell;
      margin-left: auto;
      width: 40px;
      height: 60px;
      background-color: var(--color-grey-1);
      color: var(--color-grey-3);

      svg {
        margin-top: 15px;
        height: 30px;
        width: 40px;
      }

      &:hover,
      &:focus {
        background-color: var(--color-grey-3);
        color: var(--color-grey-4);
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
</style>
