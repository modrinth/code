<template>
  <ModPage :mod="mod" :versions="versions" :members="members">
    <div class="version">
      <h3>{{ version.name }}</h3>
      <div class="markdown-body" v-html="changelog"></div>
      <hr />
      <div class="columns metadata">
        <div class="author">
          <img :src="version.author.avatar_url" />
          <p>{{ version.author.name }}</p>
        </div>
        <p>{{ version.downloads }} Downloads</p>
        <div>
          <FabricIcon
            v-if="version.loaders.includes('fabric')"
            stroke="#AC6C3A"
          />
          <ForgeIcon
            v-if="version.loaders.includes('forge')"
            stroke="#8B81E6"
          />
        </div>
        <div class="game-versions">
          <p v-for="gameVersion in version.game_versions" :key="gameVersion">
            {{ gameVersion }}
          </p>
        </div>
      </div>
      <hr />
      <div class="files">
        <div v-for="file in version.files" :key="file.hashes.sha1">
          <p>{{ file.filename }}</p>
          <a :href="file.url" download>
            <DownloadIcon />
          </a>
        </div>
      </div>
    </div>
  </ModPage>
</template>
<script>
import axios from 'axios'

import ModPage from '@/components/ModPage'
import xss from 'xss'
import marked from 'marked'

import DownloadIcon from '~/assets/images/utils/download.svg?inline'

import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  components: {
    ModPage,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
  },
  auth: false,
  async asyncData(data) {
    let res = await axios.get(
      `https://api.modrinth.com/api/v1/mod/${data.params.id}`
    )
    const mod = res.data

    res = await axios.get(
      `https://api.modrinth.com/api/v1/team/${mod.team}/members`
    )
    const members = res.data
    for (let i = 0; i < members.length; i++) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/user/${members[i].user_id}`
      )
      members[i].avatar_url = res.data.avatar_url
    }

    const versions = []
    for (const version of mod.versions) {
      res = await axios.get(
        `https://api.modrinth.com/api/v1/version/${version}`
      )

      versions.push(res.data)
    }

    const version = versions.find((x) => x.id === data.params.version)

    version.author = members.find((x) => x.user_id === version.author_id)

    res = await axios.get(version.changelog_url)
    const changelog = xss(marked(res.data))

    return {
      mod,
      versions,
      members,
      version,
      changelog,
    }
  },
  head() {
    return {
      title: this.mod.title + ' - Modrinth - Files',
      meta: [
        {
          hid: 'description',
          name: 'description',
          content:
            this.mod.description +
            ' View other minecraft mods on Modrinth today! Modrinth is a new and modern Minecraft modding platform that is compatible with CurseForge too!',
        },

        {
          hid: 'apple-mobile-web-app-title',
          name: 'apple-mobile-web-app-title',
          content: this.mod.title + ' - Modrinth',
        },
        {
          hid: 'og:site_name',
          name: 'og:site_name',
          content: this.mod.title + ' - Modrinth',
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.mod.description,
        },
        { hid: 'og:type', name: 'og:type', content: 'article' },
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
.version {
  background: var(--color-bg);
  border-radius: 0 0 0.5rem 0.5rem;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  padding: 1em;

  hr {
    margin: 20px 0;
    color: var(--color-grey-1);
  }

  .metadata {
    align-items: center;
    justify-content: space-between;

    .author {
      display: flex;
      align-items: center;
      img {
        height: 50px;
        width: 50px;
        margin-right: 10px;
      }
    }
  }

  .game-versions {
    max-width: 200px;
    p {
      margin: 0 0 0 10px;
      padding: 4px;
      font-size: 15px;
      color: var(--color-text);
      background-color: var(--color-grey-1);
      display: inline-block;
    }
  }

  .files {
    display: flex;

    div {
      display: flex;
      margin-right: 10px;
      border: 1px solid var(--color-grey-1);
      border-radius: var(--size-rounded-sm);

      p {
        margin-left: 10px;
        margin-right: 10px;
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
}
</style>
