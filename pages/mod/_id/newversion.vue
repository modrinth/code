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
            This is what users will see first. If not specified, this will
            default to the version number.
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
            This is how your version will appear in mod lists and URLs.
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
            It is important to notify players and modpack makers whether the
            version is stable or if it's still in development.
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
        <h3>Mod loaders</h3>
        <label>
          <span>Mark all mod loaders this version works with.</span>
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
        <h3>Minecraft versions</h3>
        <label>
          <span>Mark all Minecraft versions this mod version supports.</span>
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
            upload multiple.
          </span>
          <FileInput
            accept=".jar,application/java-archive,application/x-java-archive"
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
          Markdown as the description, but it is advised not to be too creative
          with the changelogs.
        </span>
        <div class="textarea-wrapper">
          <textarea v-model="createdVersion.version_body"></textarea>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
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
          data.$axios.get(`tag/loader`),
          data.$axios.get(`tag/game_version`),
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
      isEditing: true,
    }
  },
  created() {
    this.$emit('update:link-bar', [['New Version', 'newversion']])
  },
  mounted() {
    function preventLeave(e) {
      e.preventDefault()
      e.returnValue = ''
    }
    window.addEventListener('beforeunload', preventLeave)
    this.$once('hook:beforeDestroy', () => {
      window.removeEventListener('beforeunload', preventLeave)
    })
  },
  beforeRouteLeave(to, from, next) {
    if (
      this.isEditing &&
      !window.confirm('Are you sure that you want to leave without saving?')
    ) {
      return
    }
    next()
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
          await this.$axios({
            url: 'version',
            method: 'POST',
            data: formData,
            headers: {
              'Content-Type': 'multipart/form-data',
              Authorization: this.$auth.token,
            },
          })
        ).data

        this.isEditing = false
        await this.$router.push(
          `/mod/${this.mod.slug ? this.mod.slug : data.mod_id}/version/${
            data.id
          }`
        )
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
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
      await this.$axios.get(`version_file/${hash}/download`)

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
