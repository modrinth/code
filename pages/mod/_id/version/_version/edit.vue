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
            This is what users will see first. Will default to version number
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
            That's how your version will appear in mod lists and in URLs
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
            It is important to notify players and pack makers if the version is
            stable
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
        <h3>Loaders</h3>
        <label>
          <span>
            Mark all loaders this version works with. It is essential for search
          </span>
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
        <h3>Game versions</h3>
        <label>
          <span>
            Mark all game version this version supports. It is essential for
            search
          </span>
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
          markdown as description, but it is advisable not to be too creative
          with it in changelogs
        </span>
        <div class="textarea-wrapper">
          <textarea v-model="version.changelog"></textarea>
        </div>
      </div>
    </div>
  </div>
</template>
<script>
import axios from 'axios'

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
        await axios.get(this.version.changelog_url)
      ).data
    }
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
        message: 'Unable to fetch versions or loaders',
      })
    }
  },
  data() {
    return {
      version: {},
    }
  },
  mounted() {
    this.$emit('update:link-bar', [
      ['Versions', 'versions'],
      [this.version.name, 'versions/' + this.version.id],
      ['Edit Version', 'versions/' + this.version.id + '/edit'],
    ])
  },
  methods: {
    async saveVersion() {
      this.$nuxt.$loading.start()

      try {
        await axios.patch(
          `https://api.modrinth.com/api/v1/version/${this.version.id}`,
          this.version,
          this.$auth.headers
        )
        await this.$router.replace(
          `/mod/${this.mod.slug ? this.mod.slug : this.mod.id}/version/${
            this.version.id
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
