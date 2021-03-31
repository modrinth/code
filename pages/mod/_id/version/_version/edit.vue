<template>
  <ModPage
    :mod="mod"
    :versions="versions"
    :featured-versions="featuredVersions"
    :members="members"
    :current-member="currentMember"
    :link-bar="[
      ['Versions', 'versions'],
      [version.name, 'versions/' + version.id],
      ['Edit Version', 'versions/' + version.id + '/edit'],
    ]"
    :user-follows="userFollows"
  >
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
  </ModPage>
</template>
<script>
import axios from 'axios'

import Multiselect from 'vue-multiselect'
import ModPage from '~/components/layout/ModPage'

export default {
  components: {
    ModPage,
    Multiselect,
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

      const [
        members,
        versions,
        featuredVersions,
        selectableLoaders,
        selectableVersions,
        userFollows,
      ] = (
        await Promise.all([
          axios.get(`https://api.modrinth.com/api/v1/team/${mod.team}/members`),
          axios.get(`https://api.modrinth.com/api/v1/mod/${mod.id}/version`),
          axios.get(
            `https://api.modrinth.com/api/v1/mod/${mod.id}/version?featured=true`
          ),
          axios.get(`https://api.modrinth.com/api/v1/tag/loader`),
          axios.get(`https://api.modrinth.com/api/v1/tag/game_version`),
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
        selectableLoaders,
        selectableVersions,
        userFollows: userFollows.name ? null : userFollows,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Version not found',
      })
    }
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
          `/mod/${this.mod.id}/version/${this.version.id}`
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
