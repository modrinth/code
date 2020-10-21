<template>
  <div class="columns">
    <div class="content column-grow-5">
      <div class="mod-header columns">
        <img
          :src="
            mod.icon_url
              ? mod.icon_url
              : 'https://cdn.modrinth.com/placeholder.png'
          "
          alt="mod-icon"
        />
        <div class="mod-header-text">
          <div class="columns title">
            <h2>{{ mod.title }}</h2>
            <p>by Author</p>
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
        <nuxt-link :to="'/mod/' + mod.id + '/settings'">
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
      <div class="markdown-body" v-html="modBody"></div>
    </div>
    <section class="mod-info">
      <div class="mod-stats">
        <h3>Info</h3>
        <p>{{ mod.downloads }} Downloads</p>
        <p>Created {{ $dayjs(mod.published).fromNow() }}</p>
        <p>Updated {{ $dayjs(mod.updated).fromNow() }}</p>
      </div>
      <div>
        <h3>Members</h3>
        <div class="team-member columns">
          <img
            src="https://cdn.modrinth.com/placeholder.png"
            alt="profile-picture"
          />
          <div class="member-info">
            <h4>Username</h4>
            <p>Role</p>
          </div>
        </div>
      </div>
      <div>
        <h3>Featured Versions</h3>
        <div
          v-for="version in versions"
          :key="version.id"
          class="featured-version columns"
        >
          <div class="version-info">
            <div class="columns">
              <h4>{{ version.name }}</h4>
              <p v-if="version.version_type === 'release'" class="badge green">
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
              <p class="version-number">
                {{ version.version_number.substring(0, 5) }}
              </p>
              <FabricIcon
                v-if="version.loaders.includes('fabric')"
                stroke="#AC6C3A"
              />
              <ForgeIcon
                v-if="version.loaders.includes('forge')"
                stroke="#8B81E6"
              />
              <p v-if="version.game_versions.length > 0">
                {{ version.game_versions[0].substring(0, 11) }}
              </p>
            </div>
          </div>
          <nuxt-link :to="'/mod/' + mod.id + '/version/' + version.id">
            <DownloadIcon />
          </nuxt-link>
        </div>
      </div>
    </section>
  </div>
</template>

<script>
import axios from 'axios'

import xss from 'xss'
import marked from 'marked'

import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import ExternalIcon from '~/assets/images/utils/external.svg?inline'
import InfoIcon from '~/assets/images/utils/info.svg?inline'
import VersionIcon from '~/assets/images/utils/version.svg?inline'
import SettingsIcon from '~/assets/images/utils/settings.svg?inline'

import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  auth: false,
  components: {
    ExternalIcon,
    InfoIcon,
    VersionIcon,
    SettingsIcon,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
  },
  /*
{
  "id": "HKjRLvnb",
  "mod_id": "AmPXo0e2",
  "author_id": "MpxzqsyW",
  "name": "Initial Release",
  "version_number": "v1.5",
  "changelog_url": "https://cdn.modrinth.com/data/AmPXo0e2/changelogs/HKjRLvnb/body.md",
  "date_published": "2020-10-19T04:11:03.377895Z",
  "downloads": 0,
  "version_type": "release",
  "files": [],
  "dependencies": [],
  "game_versions": [
    "1.16.3",
    "1.16.2"
  ],
  "loaders": [
    "fabric"
  ]
}
  {                                                                                                                                                                                                                                                                                                                                                                           13:42:51
  id: 'kN7Mtmyo',
  team: 'eiP0Hzmw',
  title: 'Gravestones',
  description: 'A gravestones mod for fabric with tons ' +
    'of config options, an API, and more!',
  body_url: 'https://cdn.modrinth.com/file/modrinth/data/kN7Mtmyo/body.md',
  published: '2020-10-16T21:17:54.858156Z',
  updated: '2020-10-16T21:17:50.982804Z',
  status: 'processing',
  downloads: 0,
  categories: [
    'adventure',
    'utility',
    'library'
  ],
  versions: [
    'XUky61nw'
  ],
  icon_url: 'https://cdn.modrinth.com/file/modrinth/mods/icons/kN7Mtmyo/gravestones.png',
  issues_url: null,
  source_url: null,
  wiki_url: null
}
   */
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/mod/${data.params.id}`
    )
    const mod = res.data

    res = await axios.get(mod.body_url)
    const body = xss(marked(res.data))

    const versions = []

    for (const version of mod.versions) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/version/${version}`
      )

      versions.push(res.data)
    }

    return {
      mod,
      modBody: body,
      versions,
    }
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

    &.nuxt-link-active {
      border-bottom: 2px solid var(--color-brand);
    }
  }

  .filler {
    flex-grow: 1;
    border-bottom: 2px solid var(--color-grey-2);
  }
}

.markdown-body {
  border-radius: var(--size-rounded-sm);
  border-top-left-radius: 0;
  border-top-right-radius: 0;
  padding: 20px;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  background: var(--color-bg);
}

.mod-info {
  top: 1rem;
  height: calc(100vh - 2rem);
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
      margin: auto 20px;
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
    border: 1px solid var(--color-grey-1);
    border-radius: var(--size-rounded-sm);

    .version-info {
      padding: 5px 10px;
      h4 {
        font-weight: normal;
        margin: 0 10px 0 0;
      }
      .badge {
        margin: 0;
        display: inline-block;
      }
      .info-2 {
        align-items: center;

        p {
          color: var(--color-grey-4);
          font-weight: lighter;
          margin: 0 10px 0 0;
        }

        svg {
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
</style>
