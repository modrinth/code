<template>
  <div>
    <div class="new-version">
      <div class="controls">
        <button class="brand-button" title="Save version" @click="saveVersion">
          Save version
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
            v-model="version.name"
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
            v-model="version.version_number"
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
            v-model="version.version_type"
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
            v-model="version.loaders"
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
            v-model="version.game_versions"
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
      </div>
      <div class="changelog">
        <h3>Changelog</h3>
        <span>
          Tell players and modpack makers what's new. It supports the same
          Markdown as the description, but it is advised not to be too creative
          with the changelogs.
        </span>
        <div class="textarea-wrapper">
          <textarea v-model="version.changelog"></textarea>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import Multiselect from 'vue-multiselect'

export default {
  components: {
    Multiselect,
  },
  auth: false,
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
        return [{}]
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  async fetch() {
    this.version = this.versions.find(
      (x) => x.id === this.$route.params.version
    )

    if (!this.version.changelog && this.version.changelog_url) {
      this.version.changelog = (
        await this.$axios.get(this.version.changelog_url)
      ).data
    }
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
        message: 'Unable to fetch versions or loaders',
      })
    }
  },
  data() {
    return {
      version: {},
      isEditing: true,
    }
  },
  mounted() {
    this.$emit('update:link-bar', [
      ['Versions', 'versions'],
      [this.version.name, 'versions/' + this.version.id],
      ['Edit Version', 'versions/' + this.version.id + '/edit'],
    ])
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
    async saveVersion() {
      this.$nuxt.$loading.start()

      try {
        await this.$axios.patch(
          `version/${this.version.id}`,
          this.version,
          this.$auth.headers
        )
        this.isEditing = false
        await this.$router.replace(
          `/mod/${this.mod.slug ? this.mod.slug : this.mod.id}/version/${
            this.version.id
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
