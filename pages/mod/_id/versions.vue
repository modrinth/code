<template>
  <ModPage :mod="mod" :versions="versions" :members="members">
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Number</th>
          <th>Loaders</th>
          <th>Game Versions</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Published</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="version in versions" :key="version.id">
          <td>
            <nuxt-link :to="'/mod/' + mod.id + '/version/' + version.id">
              <DownloadIcon />
            </nuxt-link>
          </td>
          <td>
            <nuxt-link :to="'/mod/' + mod.id + '/version/' + version.id">
              {{ version.name }}
            </nuxt-link>
          </td>
          <td>{{ version.version_number }}</td>
          <td>
            <FabricIcon
              v-if="version.loaders.includes('fabric')"
              stroke="#AC6C3A"
            />
            <ForgeIcon
              v-if="version.loaders.includes('forge')"
              stroke="#8B81E6"
            />
          </td>
          <td>{{ version.game_versions.join(', ') }}</td>
          <td>
            <span v-if="version.version_type === 'release'" class="badge green">
              Release
            </span>
            <span v-if="version.version_type === 'beta'" class="badge yellow">
              Beta
            </span>
            <span v-if="version.version_type === 'alpha'" class="badge red">
              Alpha
            </span>
          </td>
          <td>{{ version.downloads }}</td>
          <td>{{ $dayjs(version.published).format('YYYY-MM-DD') }}</td>
        </tr>
      </tbody>
    </table>
  </ModPage>
</template>
<script>
import axios from 'axios'

import ModPage from '@/components/ModPage'

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

    return {
      mod,
      versions,
      members,
    }
  },
  head() {
    return {
      title: this.mod.title + ' - Modrinth',
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
          content: this.mod.title,
        },
        {
          hid: 'og:site_name',
          name: 'og:site_name',
          content: this.mod.title,
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
table {
  background: var(--color-bg);
  border-collapse: collapse;
  border-radius: 0 0 0.5rem 0.5rem;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  table-layout: fixed;
  width: 100%;

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-grey-2);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 5%;

      svg {
        color: var(--color-grey-3);

        &:hover,
        &:focus {
          color: var(--color-grey-5);
        }
      }
    }

    &:nth-child(2),
    &:nth-child(5) {
      padding-left: 0;
      width: 15%;
    }
  }

  th {
    color: #718096;
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 1rem 1rem;
    text-transform: uppercase;
  }

  td {
    overflow: hidden;
    padding: 0.25rem 1rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}
</style>
