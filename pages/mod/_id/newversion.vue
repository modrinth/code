<template>
  <div>
    <div class="new-version">
      <div class="controls">
        <button
          class="brand-button"
          title="Create version"
          @click="createVersion"
        >
          Create version
        </button>
      </div>
      <div class="main">
        <h3>Name</h3>
        <label>
          <span>
            This is what users will see first. Will default to version number
          </span>
          <input
            v-model="createdVersion.version_title"
            type="text"
            placeholder="Enter the name"
          />
        </label>
        <h3>Number</h3>
        <label>
          <span>
            That's how your version will appear in mod lists and in URLs
          </span>
          <input
            v-model="createdVersion.version_number"
            type="text"
            placeholder="Enter the number"
          />
        </label>
        <h3>Channel</h3>
        <label>
          <span>
            It is important to notify players and pack makers if the version is
            stable
          </span>
          <multiselect
            v-model="createdVersion.release_channel"
            placeholder="Select one"
            :options="['release', 'beta', 'alpha']"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
        </label>
        <h3>Loaders</h3>
        <label>
          <span>
            Mark all loaders this version works with. It is essential for search
          </span>
          <multiselect
            v-model="createdVersion.loaders"
            :options="selectableLoaders"
            :loading="selectableLoaders.length === 0"
            :multiple="true"
            :searchable="false"
            :show-no-results="false"
            :close-on-select="true"
            :clear-on-select="false"
            :show-labels="false"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose loaders..."
          />
        </label>
        <h3>Game versions</h3>
        <label>
          <span>
            Mark all game version this version supports. It is essential for
            search
          </span>
          <multiselect
            v-model="createdVersion.game_versions"
            :options="selectableVersions"
            :loading="selectableVersions.length === 0"
            :multiple="true"
            :searchable="true"
            :show-no-results="false"
            :close-on-select="false"
            :clear-on-select="false"
            :show-labels="false"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose versions..."
          />
        </label>
        <h3>Files</h3>
        <label>
          <span>
            You should upload a single JAR file. However, you are allowed to
            upload multiple
          </span>
          <FileInput
            accept="application/*"
            multiple
            prompt="Choose files or drag them here"
            @change="updateVersionFiles"
          />
        </label>
      </div>
      <div class="changelog">
        <h3>Changelog</h3>
        <span>
          Tell players and modpack makers what's new. It supports the same
          markdown as description, but it is advisable not to be too creative
          with it in changelogs
        </span>
        <div class="textarea-wrapper">
          <textarea v-model="createdVersion.version_body"></textarea>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import axios from 'axios'

import Multiselect from 'vue-multiselect'
import FileInput from '~/components/ui/FileInput'

export default {
  components: {
    Multiselect,
    FileInput,
  },
  props: {
    mod: {
      type: Object,
      default() {
        return {}
      },
    },
  },
  async asyncData(data) {
    try {
      const [selectableLoaders, selectableVersions] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/tag/loader`),
          axios.get(`https://api.modrinth.com/api/v1/tag/game_version`),
        ])
      ).map((it) => it.data)

      return {
        selectableLoaders,
        selectableVersions,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Unable to fetch versions and loaders',
      })
    }
  },
  data() {
    return {
      createdVersion: {},
    }
  },
  created() {
    this.$emit('update:link-bar', [['New Version', 'newversion']])
  },
  methods: {
    async createVersion() {
      this.$nuxt.$loading.start()

      const formData = new FormData()
      if (!this.createdVersion.version_title) {
        this.createdVersion.version_title = this.createdVersion.version_number
      }
      this.createdVersion.mod_id = this.mod.id
      this.createdVersion.dependencies = []
      this.createdVersion.featured = false
      formData.append('data', JSON.stringify(this.createdVersion))
      if (this.createdVersion.raw_files) {
        for (let i = 0; i < this.createdVersion.raw_files.length; i++) {
          formData.append(
            this.createdVersion.file_parts[i],
            new Blob([this.createdVersion.raw_files[i]]),
            this.createdVersion.raw_files[i].name
          )
        }
      }
      try {
        const data = (
          await axios({
            url: 'https://api.modrinth.com/api/v1/version',
            method: 'POST',
            data: formData,
            headers: {
              'Content-Type': 'multipart/form-data',
              Authorization: this.$auth.token,
            },
          })
        ).data
        await this.$router.push(
          `/mod/${this.mod.slug ? this.mod.slug : data.mod_id}/version/${
            data.id
          }`
        )
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
    updateVersionFiles(files) {
      this.createdVersion.raw_files = files

      const newFileParts = []
      for (let i = 0; i < files.length; i++) {
        newFileParts.push(files[i].name.concat('-' + i))
      }

      this.createdVersion.file_parts = newFileParts
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
.textarea-wrapper {
  display: flex;
  flex-direction: column;
  align-items: stretch;

  textarea {
    flex: 1;
    overflow-y: auto;
    resize: none;
    max-width: 100%;
  }
}

.new-version {
  @extend %card;
  padding: var(--spacing-card-md) var(--spacing-card-lg);

  display: grid;
  grid-template:
    'controls controls' auto
    'main changelog' auto
    / 5fr 4fr;
  column-gap: var(--spacing-card-md);

  .controls {
    grid-area: controls;
    display: flex;
    flex-direction: row-reverse;
  }

  .main {
    grid-area: main;
  }

  .changelog {
    grid-area: changelog;
    display: flex;
    flex-direction: column;

    .textarea-wrapper {
      flex: 1;
    }
  }
}

label {
  display: flex;

  span {
    flex: 2;
    padding-right: var(--spacing-card-lg);
  }

  input,
  .multiselect,
  .input-group {
    flex: 3;
    height: fit-content;
  }
}
</style>
