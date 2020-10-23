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
    <Popup
      v-if="
        this.$auth.loggedIn &&
        members.find((x) => x.user_id === this.$auth.user.id)
      "
      :show-popup="showPopup"
      class="create-version-popup-body"
    >
      <h3>New Version</h3>
      <div v-if="currentError" class="error">
        <h4>Error</h4>
        <p>{{ currentError }}</p>
      </div>
      <label
        for="version-title"
        class="required"
        title="The title of your version"
      >
        Version Title
      </label>
      <input
        id="version-title"
        v-model="createdVersion.version_title"
        required
        type="text"
        placeholder="Combat Update"
      />
      <label
        for="version-number"
        class="required"
        title="The version number of this version. Preferably following semantic versioning"
      >
        Version Number
      </label>
      <input
        id="version-number"
        v-model="createdVersion.version_number"
        required
        type="text"
        placeholder="v1.9"
      />
      <label class="required" title="The release channel of this version.">
        Release Channel
      </label>
      <Multiselect
        v-model="createdVersion.release_channel"
        class="categories-input"
        placeholder="Select one"
        :options="['release', 'beta', 'alpha']"
        :searchable="false"
        :close-on-select="true"
        :show-labels="false"
        :allow-empty="false"
      />
      <label
        title="The version number of this version. Preferably following semantic versioning"
      >
        Loaders
      </label>
      <multiselect
        v-model="createdVersion.loaders"
        class="categories-input"
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
      <label title="The versions of minecraft that this mod version supports">
        Game Versions
      </label>
      <multiselect
        v-model="createdVersion.game_versions"
        class="categories-input"
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
      <label for="version-body" title="A list of changes for this version">
        Changelog
      </label>
      <textarea
        id="version-body"
        v-model="createdVersion.version_body"
        class="changelog-editor"
      />
      <FileInput
        input-id="version-files"
        input-accept="application/java-archive,application/zip"
        default-text="Upload Files"
        :input-multiple="true"
        @change="updateVersionFiles"
      >
        <label class="required" title="The files associated with the version">
          Version Files
        </label>
      </FileInput>

      <div class="popup-buttons">
        <button
          class="trash-button"
          @click="
            showPopup = false
            createdVersion = {}
          "
        >
          <TrashIcon />
        </button>
        <button class="default-button" @click="createVersion">
          Create Version
        </button>
      </div>
    </Popup>
    <button class="default-button" @click="showPopup = !showPopup">
      New Version
    </button>
  </ModPage>
</template>
<script>
import axios from 'axios'

import Multiselect from 'vue-multiselect'

import ModPage from '@/components/ModPage'

import Popup from '@/components/Popup'
import FileInput from '@/components/FileInput'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import DownloadIcon from '~/assets/images/utils/download.svg?inline'
import ForgeIcon from '~/assets/images/categories/forge.svg?inline'
import FabricIcon from '~/assets/images/categories/fabric.svg?inline'

export default {
  components: {
    Multiselect,
    FileInput,
    Popup,
    ModPage,
    ForgeIcon,
    FabricIcon,
    DownloadIcon,
    TrashIcon,
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

    res = await axios.get(`https://api.modrinth.com/api/v1/tag/loader`)
    const selectableLoaders = res.data

    res = await axios.get(`https://api.modrinth.com/api/v1/tag/game_version`)
    const selectableVersions = res.data

    return {
      mod,
      versions,
      members,
      selectableLoaders,
      selectableVersions,
    }
  },
  data() {
    return {
      showPopup: false,
      currentError: null,
      createdVersion: {},
    }
  },
  methods: {
    updateVersionFiles(e) {
      this.createdVersion.raw_files = e.target.files

      const newFileParts = []
      for (let i = 0; i < e.target.files.length; i++) {
        newFileParts.push(e.target.files[i].name.concat('-' + i))
      }

      this.createdVersion.file_parts = newFileParts
    },
    async createVersion() {
      this.$nuxt.$loading.start()
      this.currentError = null

      const formData = new FormData()

      this.createdVersion.mod_id = this.$route.params.id

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
        await axios({
          url: 'https://api.modrinth.com/api/v1/version/version',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.getToken('local'),
          },
        })

        await this.$router.go(null)
      } catch (err) {
        this.currentError = err.response.data.description
        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
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
          hid: 'og:site_name',
          name: 'og:site_name',
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
.multiselect {
  margin-bottom: 20px;
}

input {
  width: calc(100% - 15px);
  padding: 0.5rem 5px;
  margin-bottom: 20px;
}

.changelog-editor {
  padding: 20px;
  width: calc(100% - 40px);
  height: 200px;
  resize: none;
  outline: none;
  border: none;
  margin: 10px 0 30px;
  background-color: var(--color-grey-1);
  color: var(--color-text);
  font-family: monospace;
}

.popup-buttons {
  margin-top: 20px;
  display: flex;
  justify-content: right;
  align-items: center;

  .default-button {
    float: none;
    margin-top: 0;
  }

  .trash-button {
    cursor: pointer;
    margin-right: 10px;
    padding: 5px;
    border: none;
    border-radius: var(--size-rounded-sm);
    color: #9b2c2c;
    background-color: var(--color-bg);
  }
}

.default-button {
  float: right;
  margin-top: 20px;
  border-radius: var(--size-rounded-sm);
  cursor: pointer;
  border: none;
  padding: 10px;
  background-color: var(--color-grey-1);
  color: var(--color-grey-5);

  &:hover,
  &:focus {
    color: var(--color-grey-4);
  }
}

.error {
  margin: 20px 0;
  border-left: #e04e3e 7px solid;
  padding: 5px 20px 20px 20px;
}
</style>
