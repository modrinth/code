<template>
  <ModPage
    :mod="mod"
    :versions="versions"
    :members="members"
    :current-member="currentMember"
  >
    <div class="version">
      <div class="version-header">
        <h4>{{ version.name }}</h4>
        <span v-if="version.version_type === 'release'" class="badge green">
          Release
        </span>
        <span v-if="version.version_type === 'beta'" class="badge yellow">
          Beta
        </span>
        <span v-if="version.version_type === 'alpha'" class="badge red">
          Alpha
        </span>
        <span>
          {{ version.version_number }}
        </span>
        <Categories :categories="version.loaders" />
        <a
          v-if="primaryFile"
          :href="primaryFile.url"
          class="download-button"
          @click.prevent="
            downloadFile(primaryFile.hashes.sha1, primaryFile.url)
          "
        >
          <DownloadIcon />
          Download
        </a>
      </div>
      <div class="stats">
        <div class="stat">
          <DownloadIcon />
          <div class="info">
            <h4>Downloads</h4>
            <p class="value">{{ version.downloads }}</p>
          </div>
        </div>
        <div class="stat">
          <CalendarIcon />
          <div class="info">
            <h4>Created</h4>
            <p
              v-tooltip="
                $dayjs(version.date_published).format(
                  '[Created on] YYYY-MM-DD [at] HH:mm A'
                )
              "
              class="value"
            >
              {{ $dayjs(version.date_published).fromNow() }}
            </p>
          </div>
        </div>
        <div class="stat">
          <TagIcon />
          <div class="info">
            <h4>Available For</h4>
            <p class="value">
              {{ version.game_versions.join(', ') }}
            </p>
          </div>
        </div>
      </div>
      <div v-compiled-markdown="changelog" class="markdown-body"></div>
      <div class="files">
        <div v-for="file in version.files" :key="file.hashes.sha1" class="file">
          <div class="text-wrapper">
            <p>{{ file.filename }}</p>
            <div v-if="currentMember" class="actions">
              <button @click="deleteFile(file.hashes.sha1)">Delete File</button>
              <button @click="makePrimary(file.hashes.sha1)">
                Make Primary
              </button>
            </div>
          </div>
          <a
            :href="file.url"
            @click.prevent="downloadFile(file.hashes.sha1, file.url)"
          >
            <DownloadIcon />
          </a>
        </div>
      </div>
      <FileInput v-if="currentMember" class="file-input" @change="addFiles" />
    </div>
  </ModPage>
</template>
<script>
import axios from 'axios'

import ModPage from '@/components/ModPage'

import Categories from '@/components/Categories'
import FileInput from '@/components/FileInput'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import TagIcon from '~/assets/images/utils/tag.svg?inline'

export default {
  components: {
    FileInput,
    Categories,
    ModPage,
    DownloadIcon,
    CalendarIcon,
    TagIcon,
  },
  auth: false,
  async fetch() {
    if (this.version.changelog_url) {
      this.changelog = (await axios.get(this.version.changelog_url)).data
    }
  },
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

      const [members, versions] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
          axios.get(
            `https://api.modrinth.com/api/v1/versions?ids=${JSON.stringify(
              mod.versions
            )}`,
            config
          ),
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

      const version = versions.find((x) => x.id === data.params.version)

      version.author = members.find((x) => x.user_id === version.author_id)

      let primaryFile = version.files.find((file) => file.primary)

      if (!primaryFile) {
        primaryFile = version.files[0]
      }

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
        version,
        primaryFile,
        currentMember,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Version not found',
      })
    }
  },
  data() {
    return {
      changelog: '',
      filesToUpload: [],
    }
  },
  methods: {
    async downloadFile(hash, url) {
      await axios.get(
        `https://api.modrinth.com/api/v1/version_file/${hash}/download`
      )

      const elem = document.createElement('a')
      elem.download = hash
      elem.href = url
      elem.click()
    },
    async deleteFile(hash) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local'),
        },
      }

      await axios.delete(
        `https://api.modrinth.com/api/v1/version_file/${hash}`,
        config
      )
    },
    async makePrimary(hash) {
      const config = {
        headers: {
          Authorization: this.$auth.getToken('local'),
        },
      }

      await axios.patch(
        `https://api.modrinth.com/api/v1/version/${this.version.id}`,
        {
          primary_file: {
            sha1: hash,
          },
        },
        config
      )
    },
    async addFiles(e) {
      this.filesToUpload = e.target.files

      for (let i = 0; i < e.target.files.length; i++) {
        this.filesToUpload[i].multipartName = e.target.files[i].name.concat(
          '-' + i
        )
      }

      this.$nuxt.$loading.start()

      const formData = new FormData()

      formData.append('data', JSON.stringify({}))

      for (const fileToUpload in this.filesToUpload) {
        formData.append(
          fileToUpload.multipartName,
          new Blob([fileToUpload]),
          fileToUpload.name
        )
      }

      try {
        await axios({
          url: `https://api.modrinth.com/api/v1/version/${this.version.id}/file`,
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.getToken('local'),
          },
        })

        await this.$router.go(null)
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An Error Occurred',
          text: err.response.data.description,
          type: 'error',
        })
        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
    },
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
.version {
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
  padding: 1rem;

  .version-header {
    display: flex;
    align-items: center;

    h4,
    span {
      margin: auto 0.5rem auto 0;
    }

    .download-button {
      margin-left: auto;
      padding: 0.5rem;
      color: var(--color-button-text);
      background-color: var(--color-button-bg);
      justify-self: flex-end;
      display: flex;
      align-items: center;
      border-radius: var(--size-rounded-sm);

      &:hover,
      &:focus {
        background-color: var(--color-button-bg-hover);
      }

      svg {
        margin-right: 0.25rem;
      }
    }
  }

  .markdown-body {
    margin: 1rem 0;
  }

  .files {
    display: flex;

    .file {
      display: flex;
      margin-right: 0.5rem;
      border-radius: var(--size-rounded-control);
      border: 1px solid var(--color-divider);

      .text-wrapper {
        display: flex;
        flex-direction: column;
        padding: 0.5rem;

        .actions {
          width: 100%;
          display: flex;
          justify-content: space-between;
          max-height: 3rem;
          font-size: var(--font-size-sm);

          button {
            display: flex;
            align-items: center;

            svg {
              margin-left: 0.25rem;
            }
          }
        }
      }

      a {
        display: flex;
        align-items: center;
        margin-left: auto;
        width: 2.5rem;
        height: auto;
        background-color: var(--color-button-bg);
        color: var(--color-button-text);
        border-radius: 0 var(--size-rounded-control) var(--size-rounded-control)
          0;

        svg {
          vertical-align: center;
          height: 30px;
          width: 40px;
        }

        &:hover,
        &:focus {
          background-color: var(--color-button-bg-hover);
          color: var(--color-button-text-hover);
        }
      }
    }
  }
}

.stats {
  display: flex;
  flex-wrap: wrap;
  margin: 0.5rem 0;
  .stat {
    margin-right: 0.75rem;
    @extend %stat;

    svg {
      padding: 0.25rem;
      border-radius: 50%;
      background-color: var(--color-button-bg);
    }
  }
}

.file-input {
  margin-top: 1rem;
}
</style>
