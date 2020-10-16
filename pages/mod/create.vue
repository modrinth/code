<template>
  <div class="content">
    <h2>Create Mod</h2>
    <section v-if="currentError" class="error">
      <h3>Error</h3>
      <p>{{ currentError }}</p>
    </section>
    <section>
      <h3>Initial Data</h3>
      <div class="initial">
        <div class="image-data">
          <img
            :src="
              previewImage
                ? previewImage
                : 'https://i0.kym-cdn.com/entries/icons/facebook/000/013/564/aP2dv.gif'
            "
          />
          <input
            id="icon-file"
            class="file-input"
            type="file"
            accept="image/x-png,image/gif,image/jpeg"
            @change="showPreviewImage"
          />
          <label for="icon-file">Upload Icon</label>
        </div>
        <div class="mod-data">
          <label for="name" class="required" title="The name of your mod">
            Name
          </label>
          <input
            id="name"
            v-model="name"
            required
            type="text"
            placeholder="Example Mod"
          />
          <label
            for="description"
            class="required"
            title="The short-form description of your mod. This shows up in searches"
          >
            Short Description
          </label>
          <input
            id="description"
            v-model="description"
            required
            type="text"
            placeholder="An example mod which does example stuff!"
          />
          <label
            for="categories"
            title="The categories of your mod, these help your mod appear in search results. Maximum of three!"
          >
            Categories
          </label>
          <multiselect
            id="categories"
            v-model="categories"
            class="categories-input"
            :options="selectableCategories"
            :loading="selectableCategories.length === 0"
            :multiple="true"
            :searchable="false"
            :show-no-results="false"
            :close-on-select="false"
            :clear-on-select="false"
            :show-labels="false"
            :max="3"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose categories..."
          />
        </div>
      </div>
    </section>
    <section class="editor">
      <h3>
        <label
          for="body"
          title="You can type the of the long form of your description here."
        >
          Mod Body
        </label>
      </h3>
      <p>
        You can type the of the long form of your description here. This editor
        supports markdown. You can find the syntax
        <a
          href="https://github.com/dimerapp/markdown/blob/develop/syntax.md"
          target="_blank"
          rel="noopener noreferrer"
          >here</a
        >.
      </p>
      <textarea id="body" v-model="body" @input="setMarkdownBody"></textarea>
      <div v-html="compiledBody"></div>
    </section>
    <section>
      <div v-if="currentVersionIndex > -1" class="create-version-popup"></div>
      <div v-if="currentVersionIndex > -1" class="create-version-popup-body">
        <div class="versions-header">
          <h3>New Version</h3>

          <div class="popup-icons">
            <TrashIcon title="Discard Version" @click="deleteVersion" />
            <SaveIcon title="Save Version" @click="currentVersionIndex = -1" />
          </div>
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
          v-model="versions[currentVersionIndex].version_title"
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
          v-model="versions[currentVersionIndex].version_number"
          required
          type="text"
          placeholder="v1.9"
        />
        <label class="required" title="The release channel of this version.">
          Release Channel
        </label>
        <Multiselect
          v-model="versions[currentVersionIndex].release_channel"
          class="categories-input"
          placeholder="Select one"
          :options="['release', 'beta', 'alpha']"
          :searchable="false"
          :close-on-select="true"
          :show-labels="false"
          :allow-empty="false"
        />
        <label
          class="required"
          title="The version number of this version. Preferably following semantic versioning"
        >
          Loaders
        </label>
        <multiselect
          v-model="versions[currentVersionIndex].loaders"
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
        <label
          class="required"
          title="The versions of minecraft that this mod version supports"
        >
          Game Versions
        </label>
        <multiselect
          v-model="versions[currentVersionIndex].game_versions"
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
          v-model="versions[currentVersionIndex].version_body"
          class="changelog-editor"
          placeholder="This editor supports markdown."
        />
        <label class="required" title="The files associated with the version">
          Version Files
        </label>
        <input
          id="version-files"
          type="file"
          accept="application/java-archive,application/zip"
          multiple
          @change="updateVersionFiles"
        />
        <label for="version-files">Upload files</label>
      </div>
      <div class="versions-header">
        <h3>Versions</h3>
        <PlusIcon @click="createVersion" />
      </div>
      <div v-for="(value, index) in versions" :key="index" class="version">
        <p>{{ value.version_number }}</p>
        <p class="column-grow-4">{{ value.version_title }}</p>
        <p>Forge</p>
        <p v-if="value.release_channel === 'beta'" class="badge yellow">Beta</p>
        <p v-if="value.release_channel === 'release'" class="badge green">
          Release
        </p>
        <p v-if="value.release_channel === 'alpha'" class="badge red">Alpha</p>
        <div>
          <TrashIcon @click="versions.splice(index, 1)" />
          <EditIcon @click="currentVersionIndex = index" />
        </div>
      </div>
    </section>
    <section>
      <h3>Extras</h3>
      <div class="extras">
        <label
          title="A link where users can go to report bugs, issues, and concerns about your mod."
        >
          Issues URL
          <input v-model="issues_url" type="text" placeholder="Optional" />
        </label>
        <label title="A link to a page/repository containing the source code ">
          Source Code Link
          <input v-model="source_url" type="text" placeholder="Optional" />
        </label>
        <label
          title="A link to a page containing information, documentation, and help for the mod. (Optional)"
        >
          Wiki Link
          <input v-model="wiki_url" type="text" placeholder="Optional" />
        </label>
      </div>
    </section>
    <button :disabled="!this.$nuxt.$loading" @click="createMod">
      Create mod
    </button>
  </div>
</template>

<script>
import axios from 'axios'
import Multiselect from 'vue-multiselect'

import DOMPurify from 'dompurify'
import marked from 'marked'

import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import EditIcon from '~/assets/images/utils/edit.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import SaveIcon from '~/assets/images/utils/save.svg?inline'

export default {
  components: {
    Multiselect,
    TrashIcon,
    EditIcon,
    PlusIcon,
    SaveIcon,
  },
  async asyncData() {
    let res = await axios.get(`https://api.modrinth.com/api/v1/tag/category`)
    const categories = res.data

    res = await axios.get(`https://api.modrinth.com/api/v1/tag/loader`)
    const loaders = res.data

    res = await axios.get(`https://api.modrinth.com/api/v1/tag/game_version`)
    const versions = res.data

    return {
      selectableCategories: categories,
      selectableLoaders: loaders,
      selectableVersions: versions,
    }
  },
  data() {
    return {
      previewImage: null,
      currentError: null,
      compiledBody: '',
      releaseChannels: ['beta', 'alpha', 'release'],
      currentVersionIndex: -1,

      name: '',
      description: '',
      body: '',
      versions: [],
      categories: [],
      issues_url: null,
      source_url: null,
      wiki_url: null,
      icon: null,
    }
  },
  methods: {
    async createMod() {
      this.$nuxt.$loading.start()
      this.currentError = null

      const formData = new FormData()

      formData.append(
        'data',
        JSON.stringify({
          mod_name: this.name,
          mod_namespace: this.namespace,
          mod_description: this.description,
          mod_body: this.body,
          initial_versions: this.versions,
          team_members: [
            {
              user_id: this.$auth.user.id,
              name: this.$auth.user.username,
              role: 'Owner',
            },
          ],
          categories: this.categories,
          issues_url: this.issues_url,
          source_url: this.source_url,
          wiki_url: this.wiki_url,
        })
      )

      if (this.icon)
        formData.append('icon', new Blob([this.icon]), this.icon.name)

      for (const version of this.versions) {
        for (let i = 0; i < version.file_parts; i++) {
          formData.append(
            version.file_parts[i],
            new Blob([version.raw_files[i]])
          )
        }
      }

      try {
        const result = await axios({
          url: 'https://api.modrinth.com/api/v1/mod',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.getToken('local'),
          },
        })

        await this.$router.replace('/dashboard/projects')

      } catch (err) {
        this.currentError = err.response.data.description
        window.scrollTo({ top: 0, behavior: 'smooth' })
        this.$nuxt.$loading.stop()
      }

      this.$nuxt.$loading.stop()
    },
    showPreviewImage(e) {
      const reader = new FileReader()
      this.icon = e.target.files[0]
      reader.readAsDataURL(this.icon)

      reader.onload = (event) => {
        this.previewImage = event.target.result
      }
    },
    updateVersionFiles(e) {
      this.versions[this.currentVersionIndex].raw_files = e.target.files

      const newFileParts = []
      for (const rawFile of e.target.files) {
        newFileParts.push(
          rawFile.name.concat(
            Math.random()
              .toString(36)
              .replace(/[^a-z]+/g, '')
              .substr(0, 5)
          )
        )
      }

      this.versions[this.currentVersionIndex].file_parts = newFileParts
    },
    createVersion() {
      this.versions.push({
        raw_files: [],
        file_parts: [],
        version_number: '',
        version_title: '',
        version_body: '',
        dependencies: [],
        game_versions: [],
        release_channel: 'release',
        loaders: [],
      })

      this.currentVersionIndex = this.versions.length - 1
    },
    deleteVersion() {
      this.versions.splice(this.currentVersionIndex, 1)
      this.currentVersionIndex = -1
    },
    setMarkdownBody() {
      this.compiledBody = DOMPurify.sanitize(marked(this.body))
    },
  },
  head() {
    return {
      bodyAttrs: {
        class: this.currentVersionIndex > -1 ? 'no-scroll' : '',
      },
    }
  },
}
</script>

<style lang="scss" scoped>
.error {
  border-left: #e04e3e 7px solid;
}

section {
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  margin: 50px 25px;
  padding: 5px 20px 20px 20px;
  border-radius: 10px;
  background-color: var(--color-bg);

  h3 {
    margin-bottom: 15px;
  }
}

input {
  width: 100%;
  padding: 0.5rem 5px;
  margin-bottom: 20px;
}

.multiselect {
  margin-top: 0.5rem;
}

[type='file'] {
  border: 0;
  clip: rect(0, 0, 0, 0);
  height: 1px;
  overflow: hidden;
  padding: 0;
  position: absolute !important;
  white-space: nowrap;
  width: 1px;

  + label {
    border-radius: 5px;
    color: var(--color-grey-5);
    background-color: var(--color-grey-1);
    padding: 10px 20px;
  }

  &:focus + label,
  + label:hover,
  &:focus + label {
    background-color: var(--color-grey-2);
    color: var(--color-text);
  }
}

.initial {
  display: flex;
  .image-data {
    flex: 1;

    img {
      image-rendering: crisp-edges;
      object-fit: cover;
      width: 100%;
      height: 85%;
      margin-bottom: 20px;
    }
  }

  .mod-data {
    flex: 4;
    margin: 20px;
  }
}

.editor {
  width: calc(100% - 90px);
  min-height: 500px;

  h3 {
    margin-bottom: 5px;
  }
  p {
    margin-top: 0;
    margin-bottom: 15px;
    a {
      text-decoration: underline;
    }
  }

  textarea {
    padding: 20px;
    width: calc(50% - 50px);
    height: 500px;
    resize: none;
    outline: none;
    border: none;
    margin: 0;
    background-color: var(--color-grey-1);
    border-right: 1px solid var(--color-text);
    color: var(--color-text);
    font-family: monospace;
  }
  div {
    padding: 20px;
    margin: 0;
    height: 540px;
    display: inline-block;
    width: calc(50%);
    box-sizing: border-box;
    vertical-align: top;
    background-color: var(--color-grey-2);
    overflow-y: auto;
    overflow-x: hidden;
  }
}

button {
  float: right;
  margin: -10px 25px 20px 0;
  cursor: pointer;
  padding: 10px;
  outline: none;
  color: var(--color-grey-5);
  background-color: var(--color-grey-1);
  border: none;
  border-radius: 5px;

  &:hover {
    background-color: var(--color-grey-2);
    color: var(--color-text);
  }
}

.extras {
  padding: 10px;
}

.required:after {
  content: ' *';
  color: red;
}

.create-version-popup {
  top: 0;
  left: 0;
  z-index: 1;
  position: fixed;
  width: 100%;
  height: 100%;
  background-color: var(--color-grey-0);
  opacity: 0.6;
  overflow-x: hidden;
}

.create-version-popup-body {
  position: fixed;
  top: 50%;
  left: 50%;
  transform: translate(-50%, -50%);
  z-index: 2;
  box-shadow: 0 2px 3px 1px var(--color-grey-2);
  padding: 5px 60px 60px 20px;
  border-radius: 10px;
  max-height: 80%;
  overflow-y: auto;
  background-color: var(--color-bg);

  .popup-icons {
    margin-top: 10px;
    margin-right: -20px;
    margin-left: auto;
  }

  .changelog-editor {
    padding: 20px;
    width: 100%;
    height: 200px;
    resize: none;
    outline: none;
    border: none;
    margin: 10px 0 30px;
    background-color: var(--color-grey-1);
    color: var(--color-text);
    font-family: monospace;
  }
}

.versions-header {
  display: flex;
  align-items: center;
}

.new-version {
  display: inline-block;
  color: var(--color-grey-5);
  background-color: var(--color-grey-1);
  border-radius: 5px;
  padding: 5px;
  cursor: pointer;
  margin-left: auto;
  float: right;

  &:hover {
    background-color: var(--color-grey-2);
    color: var(--color-text);
  }
}

.version {
  padding: 5px;
  border-radius: 5px;
  margin-bottom: 10px;
  display: flex;
  align-items: center;
  justify-content: space-around;
  background-color: var(--color-grey-1);
}

svg {
  color: var(--color-grey-5);
  cursor: pointer;

  &:hover,
  &:focus {
    color: inherit;
  }
}

.categories-input {
  margin-bottom: 15px;
}

.no-scroll {
  overflow: hidden;
}
</style>
