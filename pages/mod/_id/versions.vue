<template>
  <ModPage
    :mod="mod"
    :versions="versions"
    :members="members"
    :current-member="currentMember"
  >
    <table>
      <thead>
        <tr>
          <th></th>
          <th>Name</th>
          <th>Version</th>
          <th>Mod Loader</th>
          <th>Minecraft Version</th>
          <th>Status</th>
          <th>Downloads</th>
          <th>Date Published</th>
        </tr>
      </thead>
      <tbody>
        <tr v-for="version in versions" :key="version.id">
          <td>
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
          </td>
          <td>
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
          </td>
          <td>{{ version.version_number }}</td>
          <td>
            <FabricIcon v-if="version.loaders.includes('fabric')" />
            <ForgeIcon v-if="version.loaders.includes('forge')" />
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
          <td>{{ $dayjs(version.date_published).format('YYYY-MM-DD') }}</td>
        </tr>
      </tbody>
    </table>
    <div class="new-version">
      <nuxt-link v-if="currentMember" to="newversion" class="button">
        New Version
      </nuxt-link>
    </div>
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
    const config = {
      headers: {
        Authorization: data.$auth.getToken('local')
          ? data.$auth.getToken('local')
          : '',
      },
    }

    try {
      const mod = (
        await axios.get(
          `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
          config
        )
      ).data

      const [members, versions, selectableLoaders, selectableVersions] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
          axios.get(
            `https://api.modrinth.com/api/v1/versions?ids=${JSON.stringify(
              mod.versions
            )}`,
            config
          ),
          axios.get(`https://api.modrinth.com/api/v1/tag/loader`),
          axios.get(`https://api.modrinth.com/api/v1/tag/game_version`),
        ])
      ).map((it) => it.data)

      const users = (
        await axios.get(
          `https://api.modrinth.com/api/v1/users?ids=${JSON.stringify(
            members.map((it) => it.user_id)
          )}`,
          config
        )
      ).data

      users.forEach((it) => {
        const index = members.findIndex((x) => x.user_id === it.id)
        members[index].avatar_url = it.avatar_url
        members[index].name = it.username
      })

      const currentMember = data.$auth.loggedIn
        ? members.find((x) => x.user_id === data.$auth.user.id)
        : null

      return {
        mod,
        versions: versions.sort(
          (a, b) =>
            new Date(b.date_published).getTime() -
            new Date(a.date_published).getTime()
        ),
        members,
        selectableLoaders,
        selectableVersions,
        currentMember,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Mod not found',
      })
    }
  },
  methods: {
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
          hid: 'og:title',
          name: 'og:title',
          content: this.mod.title,
        },
        {
          hid: 'og:url',
          name: 'og:url',
          content: `https://modrinth.com/mod/${this.mod.id}`,
        },
        {
          hid: 'og:description',
          name: 'og:description',
          content: this.mod.description,
        },
        { hid: 'og:type', name: 'og:type', content: 'website' },
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
  border-collapse: collapse;
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
  table-layout: fixed;
  width: 100%;

  * {
    text-align: left;
  }

  tr:not(:last-child),
  tr:first-child {
    th,
    td {
      border-bottom: 1px solid var(--color-divider);
    }
  }

  th,
  td {
    &:first-child {
      text-align: center;
      width: 7%;

      svg {
        color: var(--color-text);

        &:hover,
        &:focus {
          color: var(--color-text-hover);
        }
      }
    }

    &:nth-child(2),
    &:nth-child(5) {
      padding-left: 0;
      width: 12%;
    }
  }

  th {
    color: var(--color-heading);
    font-size: 0.8rem;
    letter-spacing: 0.02rem;
    margin-bottom: 0.5rem;
    margin-top: 1.5rem;
    padding: 0.75rem 1rem;
    text-transform: uppercase;
  }

  td {
    overflow: hidden;
    padding: 0.75rem 1rem;

    img {
      height: 3rem;
      width: 3rem;
    }
  }
}

.new-version {
  width: 100%;
  text-align: right;
  margin-bottom: var(--spacing-card-md);
}

@media screen and (max-width: 400px) {
  th,
  td {
    &:nth-child(7) {
      display: none;
    }
  }
}

@media screen and (max-width: 600px) {
  th,
  td {
    &:nth-child(8) {
      display: none;
    }
  }
}

@media screen and (max-width: 800px) {
  th,
  td {
    &:nth-child(5) {
      display: none;
    }
  }
}

@media screen and (max-width: 1000px) {
  th,
  td {
    &:nth-child(2) {
      display: none;
    }
  }
}
</style>
