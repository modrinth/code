<template>
  <ModPage
    :mod="mod"
    :versions="versions"
    :members="members"
    :current-member="currentMember"
    :featured-versions="featuredVersions"
    :link-bar="[
      ['Versions', 'versions'],
      [version.name, 'versions/' + version.id],
    ]"
    :user-follows="userFollows"
  >
    <ConfirmPopup
      ref="delete_file_popup"
      title="Are you sure you want to delete this file?"
      description="This will remove this file forever (like really forever)"
      :has-to-type="false"
      proceed-label="Delete File"
      @proceed="deleteFile(popup_data)"
    />
    <ConfirmPopup
      ref="delete_version_popup"
      title="Are you sure you want to delete this version?"
      description="This will remove this version forever (like really forever), and if some mods depends on this version, it won't work anymore."
      :has-to-type="false"
      proceed-label="Delete Version"
      @proceed="deleteVersion()"
    />
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
        <div class="buttons">
          <nuxt-link
            v-if="this.$auth.user"
            :to="`/report/create?id=${version.id}&t=version`"
            class="action iconified-button"
          >
            <ReportIcon />
            Report
          </nuxt-link>
          <button
            v-if="currentMember"
            class="action iconified-button"
            @click="deleteVersionPopup"
          >
            <TrashIcon />
            Delete
          </button>
          <nuxt-link
            v-if="currentMember"
            class="action iconified-button"
            :to="version.id + '/edit'"
          >
            <EditIcon />
            Edit
          </nuxt-link>
          <a
            v-if="primaryFile"
            :href="primaryFile.url"
            class="action iconified-button"
            @click.prevent="
              downloadFile(primaryFile.hashes.sha1, primaryFile.url)
            "
          >
            <DownloadIcon />
            Download
          </a>
        </div>
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
      <div
        v-compiled-markdown="version.changelog ? version.changelog : ''"
        class="markdown-body"
      ></div>
      <div class="files">
        <div v-for="file in version.files" :key="file.hashes.sha1" class="file">
          <div class="text-wrapper">
            <p>{{ file.filename }}</p>
            <div v-if="currentMember" class="actions">
              <button @click="deleteFilePopup(file.hashes.sha1)">
                Delete File
              </button>
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

import ModPage from '~/components/layout/ModPage'
import ConfirmPopup from '~/components/ui/ConfirmPopup'

import Categories from '~/components/ui/search/Categories'
import FileInput from '~/components/ui/FileInput'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import CalendarIcon from '~/assets/images/utils/calendar.svg?inline'
import TagIcon from '~/assets/images/utils/tag.svg?inline'
import ReportIcon from '~/assets/images/utils/report.svg?inline'

export default {
  components: {
    FileInput,
    Categories,
    ModPage,
    DownloadIcon,
    CalendarIcon,
    TagIcon,
    TrashIcon,
    EditIcon,
    ReportIcon,
    ConfirmPopup,
  },
  auth: false,
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

      const version = versions.find((x) => x.id === data.params.version)

      version.author = members.find((x) => x.user_id === version.author_id)

      let primaryFile = version.files.find((file) => file.primary)

      if (!primaryFile) {
        primaryFile = version.files[0]
      }

      const currentMember = data.$auth.user
        ? members.find((x) => x.user_id === data.$auth.user.id)
        : null

      if (!version.changelog && version.changelog_url) {
        version.changelog = (await axios.get(version.changelog_url)).data
      }

      return {
        mod,
        versions,
        featuredVersions,
        members,
        version,
        primaryFile,
        currentMember,
        userFollows: userFollows.name ? null : userFollows,
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
      filesToUpload: [],
      popup_data: null,
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
    deleteFilePopup(hash) {
      this.popup_data = hash
      this.$refs.delete_file_popup.show()
    },
    async deleteFile(hash) {
      this.$nuxt.$loading.start()

      await axios.delete(
        `https://api.modrinth.com/api/v1/version_file/${hash}`,
        this.$auth.headers
      )

      await this.$router.go(null)
      this.$nuxt.$loading.finish()
    },
    async makePrimary(hash) {
      this.$nuxt.$loading.start()

      await axios.patch(
        `https://api.modrinth.com/api/v1/version/${this.version.id}`,
        {
          primary_file: ['sha1', hash],
        },
        this.$auth.headers
      )

      await this.$router.go(null)
      this.$nuxt.$loading.finish()
    },
    async addFiles(files) {
      this.filesToUpload = files

      for (let i = 0; i < files.length; i++) {
        this.filesToUpload[i].multipartName = files[i].name.concat('-' + i)
      }

      this.$nuxt.$loading.start()

      const formData = new FormData()

      formData.append('data', JSON.stringify({}))

      for (const fileToUpload of this.filesToUpload) {
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
            Authorization: this.$auth.token,
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
    deleteVersionPopup() {
      this.$refs.delete_version_popup.show()
    },
    async deleteVersion() {
      this.$nuxt.$loading.start()

      await axios.delete(
        `https://api.modrinth.com/api/v1/version/${this.version.id}`,
        this.$auth.headers
      )

      await this.$router.replace(`/mod/${this.mod.id}`)
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
    flex-wrap: wrap;

    h4,
    span {
      margin: auto 0.5rem auto 0;
    }

    .buttons {
      display: flex;
      align-self: flex-end;

      @media screen and (min-width: 1024px) {
        margin-left: auto;
      }

      .action:not(:first-child) {
        margin: 0 0 0 0.5rem;
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
