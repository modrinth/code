<template>
  <div class="page-container">
    <div class="page-contents">
      <header class="card columns">
        <h3 class="column-grow-1">Create a project</h3>
        <button
          title="Save draft"
          class="iconified-button column"
          :disabled="!$nuxt.$loading"
          @click="createDraft"
        >
          Save draft
        </button>
        <button
          title="Submit for review"
          class="iconified-button brand-button-colors column"
          :disabled="!$nuxt.$loading"
          @click="createProject"
        >
          <CheckIcon />
          Submit for review
        </button>
      </header>
      <section class="card essentials">
        <h3>Project type</h3>
        <label>
          <span class="no-padding">The project type of your project.</span>
          <Multiselect
            v-model="projectType"
            placeholder="Select one"
            :options="projectTypes"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
            :allow-empty="false"
          />
        </label>
        <h3>Name</h3>
        <label>
          <span>
            Be creative! Generic project names will be harder to search for.
          </span>
          <input v-model="name" type="text" placeholder="Enter the name" />
        </label>
        <h3>Summary</h3>
        <label>
          <span>
            Give a short description of your project that will appear on search
            pages.
          </span>
          <input
            v-model="description"
            type="text"
            placeholder="Enter the summary"
          />
        </label>
        <h3>Categories</h3>
        <label>
          <span class="no-padding">
            Select up to 3 categories that will help others <br />
            find your project.
          </span>
          <multiselect
            id="categories"
            v-model="categories"
            :options="
              $tag.categories
                .filter((x) => x.project_type === projectType.toLowerCase())
                .map((it) => it.name)
            "
            :loading="$tag.categories.length === 0"
            :multiple="true"
            :searchable="false"
            :show-no-results="false"
            :close-on-select="false"
            :clear-on-select="false"
            :show-labels="false"
            :max="3"
            :limit="6"
            :hide-selected="true"
            placeholder="Choose categories"
          />
        </label>
        <h3>Vanity URL (slug)</h3>
        <label>
          <span>
            Set this to something that will looks nice in your project's URL.
          </span>
          <input
            id="name"
            v-model="slug"
            type="text"
            placeholder="Enter the vanity URL slug"
          />
        </label>
      </section>
      <section class="card project-icon">
        <h3>Icon</h3>
        <img
          :src="
            previewImage
              ? previewImage
              : 'https://cdn.modrinth.com/placeholder.svg'
          "
          alt="preview-image"
        />
        <file-input
          accept="image/png,image/jpeg,image/gif,image/webp"
          class="choose-image"
          prompt="Choose image or drag it here"
          @change="showPreviewImage"
        />
        <button
          class="iconified-button"
          @click="
            icon = null
            previewImage = null
          "
        >
          <TrashIcon />
          Reset
        </button>
      </section>
      <section class="card game-sides">
        <h3>Supported environments</h3>
        <div class="columns">
          <span>
            Let others know what environments your project supports.
          </span>
          <div class="labeled-control">
            <h3>Client</h3>
            <Multiselect
              v-model="clientSideType"
              placeholder="Select one"
              :options="sideTypes"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
          </div>
          <div class="labeled-control">
            <h3>Server</h3>
            <Multiselect
              v-model="serverSideType"
              placeholder="Select one"
              :options="sideTypes"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
          </div>
        </div>
      </section>
      <section class="card description">
        <h3>
          <label
            for="body"
            title="You can type an extended description of your project here."
          >
            Body
          </label>
        </h3>
        <span>
          You can type an extended description of your mod here. This editor
          supports Markdown. Its syntax can be found
          <a
            class="text-link"
            href="https://guides.github.com/features/mastering-markdown/"
            target="_blank"
            rel="noopener noreferrer"
            >here</a
          >. HTML can also be used inside your description, excluding scripts
          and iframes.
        </span>
        <ThisOrThat
          v-model="bodyViewMode"
          class="separator"
          :items="['source', 'preview']"
        />
        <div class="edit-wrapper">
          <div v-if="bodyViewMode === 'source'" class="textarea-wrapper">
            <textarea id="body" v-model="body" />
          </div>
          <div
            v-if="bodyViewMode === 'preview'"
            v-highlightjs
            class="markdown-body"
            v-html="body ? $xss($md.render(body)) : 'No body specified.'"
          ></div>
        </div>
      </section>
      <section class="card versions">
        <div class="title">
          <h3>Create versions</h3>
          <button
            title="Add a version"
            class="iconified-button"
            :disabled="currentVersionIndex !== -1"
            @click="createVersion"
          >
            <PlusIcon />
            Add a version
          </button>
        </div>
        <table>
          <thead>
            <tr>
              <th>Name</th>
              <th>Version</th>
              <th>Mod loader</th>
              <th>Minecraft version</th>
              <th>Channel</th>
              <th>Actions</th>
            </tr>
          </thead>
          <tbody>
            <tr
              v-for="(version, index) in versions.filter((it) =>
                currentVersionIndex !== -1
                  ? it !== versions[currentVersionIndex]
                  : true
              )"
              :key="version.id"
            >
              <td>{{ version.version_title }}</td>
              <td>{{ version.version_number }}</td>
              <td>
                {{
                  version.loaders
                    .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                    .join(', ')
                }}
              </td>
              <td>
                {{
                  version.game_versions
                    .map((x) => x.charAt(0).toUpperCase() + x.slice(1))
                    .join(', ')
                }}
              </td>
              <td>
                <VersionBadge
                  v-if="version.release_channel === 'release'"
                  type="release"
                  color="green"
                />
                <VersionBadge
                  v-else-if="version.release_channel === 'beta'"
                  type="beta"
                  color="yellow"
                />
                <VersionBadge
                  v-else-if="version.release_channel === 'alpha'"
                  type="alpha"
                  color="red"
                />
              </td>
              <td>
                <button
                  class="iconified-button"
                  title="Remove version"
                  @click="versions.splice(index, 1)"
                >
                  Remove
                </button>
                <button
                  class="iconified-button"
                  title="Edit version"
                  @click="currentVersionIndex = index"
                >
                  Edit
                </button>
              </td>
            </tr>
          </tbody>
        </table>
        <hr v-if="currentVersionIndex !== -1" />
        <div v-if="currentVersionIndex !== -1" class="new-version">
          <div class="controls">
            <button
              class="brand-button-colors iconified-button"
              title="Save version"
              @click="currentVersionIndex = -1"
            >
              Save version
            </button>
            <button
              class="iconified-button"
              title="Discard version"
              @click="deleteVersion"
            >
              Discard
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
                v-model="versions[currentVersionIndex].version_title"
                type="text"
                placeholder="Enter the name"
              />
            </label>
            <h3>Number</h3>
            <label>
              <span>
                This is how your version will appear in project lists and URLs.
              </span>
              <input
                v-model="versions[currentVersionIndex].version_number"
                type="text"
                placeholder="Enter the number"
              />
            </label>
            <h3>Channel</h3>
            <label>
              <span>
                It is important to notify everyone whether the version is stable
                or if it's still in development.
              </span>
              <multiselect
                v-model="versions[currentVersionIndex].release_channel"
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
              <span> Mark all mod loaders this version works with. </span>
              <multiselect
                v-model="versions[currentVersionIndex].loaders"
                :options="
                  $tag.loaders
                    .filter((x) =>
                      x.supported_project_types.includes(
                        projectType.toLowerCase()
                      )
                    )
                    .map((it) => it.name)
                "
                :loading="$tag.loaders.length === 0"
                :multiple="true"
                :searchable="false"
                :show-no-results="false"
                :close-on-select="true"
                :clear-on-select="false"
                :show-labels="false"
                :limit="6"
                :hide-selected="true"
                placeholder="Choose mod loaders..."
              />
            </label>
            <h3>Minecraft versions</h3>
            <label>
              <span>
                Mark all Minecraft version this project's version supports.
              </span>
              <multiselect
                v-model="versions[currentVersionIndex].game_versions"
                :options="$tag.gameVersions.map((it) => it.version)"
                :loading="$tag.gameVersions.length === 0"
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
          <div class="dependencies">
            <h3>Dependencies</h3>
            <div class="dependency-selector">
              <ThisOrThat
                v-model="dependencyAddMode"
                class="separator"
                :items="['project', 'version']"
              />
              <div class="edit-info">
                <input
                  v-model="newDependencyId"
                  type="text"
                  :placeholder="`Enter the ${dependencyAddMode} ID...`"
                />
                <Multiselect
                  v-model="newDependencyType"
                  class="input"
                  placeholder="Select one"
                  :options="['required', 'optional', 'incompatible']"
                  :searchable="false"
                  :close-on-select="true"
                  :show-labels="false"
                  :allow-empty="false"
                />
                <button class="iconified-button" @click="addDependency">
                  <PlusIcon />
                  Add
                </button>
              </div>
            </div>
            <div class="new-dependencies">
              <div
                v-for="(dependency, index) in versions[currentVersionIndex]
                  .dependencies"
                :key="index"
                class="dependency"
              >
                <img
                  class="icon"
                  :src="
                    dependency.project
                      ? dependency.project.icon_url
                        ? dependency.project.icon_url
                        : 'https://cdn.modrinth.com/placeholder.svg?inline'
                      : 'https://cdn.modrinth.com/placeholder.svg?inline'
                  "
                  alt="dependency-icon"
                />
                <div class="info">
                  <h4 class="title">
                    {{
                      dependency.project
                        ? dependency.project.title
                        : 'Unknown Project'
                    }}
                  </h4>
                  <p v-if="dependency.version" class="version-number">
                    Version {{ dependency.version.version_number }} is
                    {{ dependency.dependency_type }}
                  </p>
                  <div class="bottom">
                    <button
                      class="iconified-button"
                      @click="
                        versions[currentVersionIndex].dependencies.splice(
                          index,
                          1
                        )
                      "
                    >
                      <TrashIcon /> Delete
                    </button>
                  </div>
                </div>
              </div>
            </div>
          </div>
          <div class="files">
            <h3>Files</h3>
            <SmartFileInput
              class="file-input"
              multiple
              accept=".jar,application/java-archive,.zip,application/zip,.mrpack"
              prompt="Upload files"
              @change="
                (x) =>
                  x.forEach((y) => versions[currentVersionIndex].files.push(y))
              "
            />
            <div class="uploaded-files">
              <div
                v-for="(file, index) in versions[currentVersionIndex].files"
                :key="index + 'new'"
                class="file"
              >
                <p class="filename">{{ file.name }}</p>
                <button
                  class="action iconified-button"
                  @click="versions[currentVersionIndex].files.splice(index, 1)"
                >
                  <TrashIcon aria-hidden="true" />
                  Delete
                </button>
              </div>
            </div>
          </div>
          <div class="changelog">
            <h3>Changes</h3>
            <span>
              Tell everyone what's new. It supports the same Markdown formatting
              as the description.
            </span>
            <ThisOrThat
              v-model="changelogViewMode"
              class="separator"
              :items="['source', 'preview']"
            />
            <div class="edit-wrapper">
              <div
                v-if="changelogViewMode === 'source'"
                class="textarea-wrapper"
              >
                <textarea
                  id="body"
                  v-model="versions[currentVersionIndex].version_body"
                />
              </div>
              <div
                v-if="changelogViewMode === 'preview'"
                v-highlightjs
                class="markdown-body"
                v-html="
                  versions[currentVersionIndex].version_body
                    ? $xss(
                        $md.render(versions[currentVersionIndex].version_body)
                      )
                    : 'No changelog specified.'
                "
              ></div>
            </div>
          </div>
        </div>
      </section>
      <section class="card gallery">
        <div class="title">
          <div class="text">
            <h3>Gallery</h3>
            <i>— this section is optional</i>
          </div>
          <button
            title="Add an image"
            class="iconified-button"
            :disabled="false"
            @click="gallery.push({})"
          >
            <PlusIcon />
            Add an image
          </button>
        </div>
        <div v-for="(item, index) in gallery" :key="index" class="gallery-item">
          <div class="info">
            <label title="The title of the gallery item">
              <span>Title</span>
              <input
                v-model="item.title"
                type="text"
                placeholder="Enter a title"
              />
            </label>
            <label
              title="The description of what is being featured in the link."
            >
              <span>Description (optional)</span>
              <input
                v-model="item.description"
                type="text"
                placeholder="Enter a description"
              />
            </label>
          </div>
          <div class="image">
            <img
              :src="
                item.preview
                  ? item.preview
                  : 'https://cdn.modrinth.com/placeholder-banner.svg'
              "
              alt="preview-image"
            />
            <div class="bottom">
              <SmartFileInput
                accept="image/png,image/jpeg,image/gif,image/webp,.png,.jpeg,.gif,.webp"
                prompt="Upload"
                @change="(files) => showGalleryPreviewImage(files, index)"
              />
              <button
                class="iconified-button"
                @click="
                  item.icon = null
                  item.preview = null
                  $forceUpdate()
                "
              >
                <TrashIcon />
                Reset
              </button>
            </div>
          </div>
          <div class="buttons">
            <button class="iconified-button" @click="gallery.splice(index, 1)">
              <TrashIcon />
              Remove
            </button>

            <hr v-if="gallery.length > 0 && index !== gallery.length - 1" />
          </div>
        </div>
      </section>
      <section class="card extra-links">
        <div class="title">
          <div class="text">
            <h3>External links</h3>
            <i>— this section is optional</i>
          </div>
        </div>
        <label
          title="A place for users to report bugs, issues, and concerns about your project."
        >
          <span>Issue tracker</span>
          <input
            v-model="issues_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label
          title="A page/repository containing the source code for your project"
        >
          <span>Source code</span>
          <input
            v-model="source_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label
          title="A page containing information, documentation, and help for the project."
        >
          <span>Wiki page</span>
          <input
            v-model="wiki_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
        <label title="An invitation link to your Discord server.">
          <span>Discord invite</span>
          <input
            v-model="discord_url"
            type="url"
            placeholder="Enter a valid URL"
          />
        </label>
      </section>
      <section class="card license">
        <div class="title">
          <div class="text">
            <h3>License</h3>
          </div>
        </div>
        <label>
          <span>
            It is very important to choose a proper license for your mod. You
            may choose one from our list or provide a URL to a custom license.
            <br />
            Confused? See our
            <a
              class="text-link"
              href="https://blog.modrinth.com/licensing-guide/"
              target="_blank"
              rel="noopener noreferrer"
            >
              licensing guide</a
            >
            for more information.
          </span>
          <div class="input-group">
            <Multiselect
              v-model="license"
              placeholder="Choose license..."
              :loading="$tag.licenses.length === 0"
              :options="$tag.licenses"
              track-by="short"
              label="short"
              :multiple="false"
              :searchable="true"
              :close-on-select="true"
              :show-labels="false"
              :allow-empty="false"
            />
            <input v-model="license_url" type="url" placeholder="License URL" />
          </div>
        </label>
      </section>
      <section class="card donations">
        <div class="title">
          <div class="text">
            <h3>Donation links</h3>
            <i>— this section is optional</i>
          </div>
          <button
            title="Add a link"
            class="iconified-button"
            :disabled="false"
            @click="
              donationPlatforms.push({})
              donationLinks.push('')
            "
          >
            <PlusIcon />
            Add a link
          </button>
        </div>
        <div v-for="(item, index) in donationPlatforms" :key="index">
          <label title="The donation link.">
            <span>Donation Link</span>
            <input
              v-model="donationLinks[index]"
              type="url"
              placeholder="Enter a valid URL"
            />
          </label>
          <label title="The donation platform of the link.">
            <span>Donation Platform</span>
            <Multiselect
              v-model="donationPlatforms[index]"
              placeholder="Select one"
              track-by="short"
              label="name"
              :options="$tag.donationPlatforms"
              :searchable="false"
              :close-on-select="true"
              :show-labels="false"
            />
          </label>
          <button
            class="iconified-button"
            @click="
              donationPlatforms.splice(index, 1)
              donationLinks.splice(index, 1)
            "
          >
            <TrashIcon />
            Remove Link
          </button>
          <hr
            v-if="
              donationPlatforms.length > 0 &&
              index !== donationPlatforms.length - 1
            "
          />
        </div>
      </section>
    </div>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import FileInput from '~/components/ui/FileInput'
import SmartFileInput from '~/components/ui/SmartFileInput'
import ThisOrThat from '~/components/ui/ThisOrThat'
import VersionBadge from '~/components/ui/Badge'

import CheckIcon from '~/assets/images/utils/check.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'

export default {
  components: {
    FileInput,
    SmartFileInput,
    Multiselect,
    CheckIcon,
    PlusIcon,
    TrashIcon,
    ThisOrThat,
    VersionBadge,
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
  data() {
    return {
      previewImage: null,
      compiledBody: '',
      releaseChannels: ['beta', 'alpha', 'release'],
      currentVersionIndex: -1,
      name: '',
      slug: '',
      draft: false,
      description: '',
      body: '',
      versions: [],
      categories: [],
      issues_url: null,
      source_url: null,
      wiki_url: null,
      discord_url: null,
      icon: null,
      license: null,
      license_url: null,

      projectTypes: ['Mod', 'Modpack'],
      projectType: 'Mod',

      sideTypes: ['Required', 'Optional', 'Unsupported'],
      clientSideType: 'Required',
      serverSideType: 'Required',

      donationLinks: [],
      donationPlatforms: [],

      gallery: [],

      isEditing: true,
      bodyViewMode: 'source',
      changelogViewMode: 'source',

      newDependencyType: 'required',
      dependencyAddMode: 'project',
      newDependencyId: '',
    }
  },
  watch: {
    license(newValue, oldValue) {
      if (newValue == null) {
        this.license_url = ''
        return
      }

      switch (newValue.short) {
        case 'custom':
          this.license_url = ''
          break
        default:
          this.license_url = `https://cdn.modrinth.com/licenses/${newValue.short}.txt`
      }
    },
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
  methods: {
    async createDraft() {
      this.draft = true
      await this.createProject()
    },
    async createProject() {
      this.$nuxt.$loading.start()

      for (let i = 0; i < this.versions.length; i++) {
        const version = this.versions[i]
        if (!version.version_title) {
          version.version_title = version.version_number
        }

        const newFileParts = []
        for (let j = 0; j < version.files.length; j++) {
          newFileParts.push(`version-${i}-${j}`)
        }

        version.file_parts = newFileParts
      }

      const formData = new FormData()

      formData.append(
        'data',
        JSON.stringify({
          title: this.name,
          project_type: this.projectType.toLowerCase(),
          slug: this.slug,
          description: this.description,
          body: this.body,
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
          discord_url: this.discord_url,
          client_side: this.clientSideType.toLowerCase(),
          server_side: this.serverSideType.toLowerCase(),
          license_id: this.license ? this.license.short : 'arr',
          license_url: this.license_url,
          is_draft: this.draft,
          donation_urls: this.donationPlatforms.map((it, index) => {
            return {
              id: it.short,
              platform: it.name,
              url: this.donationLinks[index],
            }
          }),
          gallery_items: this.gallery.map((x, index) => {
            return {
              item: `gallery-${index}`,
              featured: false,
              title: x.title,
              description: x.description,
            }
          }),
        })
      )

      if (this.icon) {
        formData.append('icon', new Blob([this.icon]), this.icon.name)
      }

      for (let i = 0; i < this.gallery.length; i++) {
        formData.append(
          `gallery-${i}`,
          new Blob([this.gallery[i].icon]),
          this.gallery[i].icon.name
        )
      }

      for (let i = 0; i < this.versions.length; i++) {
        const version = this.versions[i]
        for (let j = 0; j < version.files.length; j++) {
          formData.append(
            `version-${i}-${j}`,
            new Blob([version.files[j]]),
            version.files[j].name
          )
        }
      }

      try {
        await this.$axios({
          url: 'project',
          method: 'POST',
          data: formData,
          headers: {
            'Content-Type': 'multipart/form-data',
            Authorization: this.$auth.token,
          },
        })

        this.isEditing = false
        await this.$store.dispatch('user/fetchProjects')

        await this.$router.replace(`/user/${this.$auth.user.username}`)
      } catch (err) {
        let description = err.response.data.description

        if (description.includes('JSON')) {
          description = 'Please fill in missing required fields.'
        }

        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: description,
          type: 'error',
          duration: 10000,
        })

        window.scrollTo({ top: 0, behavior: 'smooth' })
      }

      this.$nuxt.$loading.finish()
    },

    showPreviewImage(files) {
      const reader = new FileReader()
      this.icon = files[0]

      if (this.icon instanceof Blob) {
        reader.readAsDataURL(this.icon)

        reader.onload = (event) => {
          this.previewImage = event.target.result
        }
      }
    },

    showGalleryPreviewImage(files, index) {
      const reader = new FileReader()
      this.gallery[index].icon = files[0]

      if (this.gallery[index].icon instanceof Blob) {
        reader.readAsDataURL(this.gallery[index].icon)

        reader.onload = (event) => {
          this.gallery[index].preview = event.target.result

          // TODO: Find an alternative for this!
          this.$forceUpdate()
        }
      }
    },

    createVersion() {
      this.versions.push({
        files: [],
        version_number: '',
        version_title: '',
        version_body: '',
        dependencies: [],
        game_versions: [],
        release_channel: 'release',
        loaders: [],
        featured: false,
      })

      this.currentVersionIndex = this.versions.length - 1
    },

    deleteVersion() {
      this.versions.splice(this.currentVersionIndex, 1)
      this.currentVersionIndex = -1
    },

    async addDependency() {
      try {
        if (this.dependencyAddMode === 'project') {
          const project = (
            await this.$axios.get(`project/${this.newDependencyId}`)
          ).data

          this.versions[this.currentVersionIndex].dependencies.push({
            project,
            project_id: project.id,
            dependency_type: this.newDependencyType,
          })
        } else if (this.dependencyAddMode === 'version') {
          const version = (
            await this.$axios.get(`version/${this.newDependencyId}`)
          ).data
          const project = (
            await this.$axios.get(`project/${version.project_id}`)
          ).data

          this.versions[this.currentVersionIndex].dependencies.push({
            version,
            project,
            version_id: version.id,
            project_id: project.id,
            dependency_type: this.newDependencyType,
          })
        }
      } catch {
        this.$notify({
          group: 'main',
          title: 'Invalid Dependency',
          text: 'The specified dependency does not exist',
          type: 'error',
        })
      }

      this.newDependencyId = ''
    },
  },
}
</script>

<style lang="scss" scoped>
.title {
  .text * {
    display: inline;
  }

  display: flex;
  justify-content: space-between;
  align-items: center;
}

.page-contents {
  display: grid;
  grid-template:
    'header       header       header' auto
    'essentials   essentials   essentials' auto
    'project-icon project-icon project-icon' auto
    'game-sides   game-sides   game-sides' auto
    'description  description  description' auto
    'versions     versions     versions' auto
    'gallery      gallery      gallery' auto
    'extra-links  extra-links  extra-links' auto
    'license      license      license' auto
    'donations    donations    donations' auto
    'footer       footer       footer' auto
    / 4fr 1fr 4fr;

  @media screen and (min-width: 1024px) {
    grid-template:
      'header       header      header' auto
      'essentials   essentials  project-icon' auto
      'game-sides   game-sides  game-sides' auto
      'description  description description' auto
      'versions     versions    versions' auto
      'gallery      gallery      gallery' auto
      'extra-links  license     license' auto
      'donations    donations   donations' auto
      'footer       footer      footer' auto
      / 4fr 2fr 1.5fr;
  }

  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-md);
}

header {
  grid-area: header;

  h3 {
    margin: auto 0;
    color: var(--color-text-dark);
    font-weight: var(--font-weight-extrabold);
  }

  button {
    margin-left: 0.5rem;
  }
}

section.essentials {
  grid-area: essentials;
}

section.project-icon {
  grid-area: project-icon;

  img {
    max-width: 100%;
    margin-bottom: 1rem;
    border-radius: var(--size-rounded-lg);
  }

  .iconified-button {
    margin-top: 0.5rem;
  }
}

section.game-sides {
  grid-area: game-sides;

  .columns {
    flex-wrap: wrap;

    span {
      flex: 2;
    }

    .labeled-control {
      flex: 2;
      margin-left: var(--spacing-card-lg);

      h3 {
        margin-bottom: var(--spacing-card-sm);
      }
    }
  }
}

section.description {
  grid-area: description;

  .separator {
    margin: var(--spacing-card-sm) 0;
  }

  .edit-wrapper * {
    min-height: 10rem;
    max-height: 40rem;
  }

  .markdown-body {
    overflow-y: auto;
    padding: 0 var(--spacing-card-sm);
  }
}

section.versions {
  grid-area: versions;

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

      &:last-child {
        display: flex;
      }

      @media screen and (max-width: 800px) {
        + &:nth-child(4),
        &:nth-child(3) {
          display: none;
        }
        &:first-child,
        &:nth-child(5) {
          width: unset;
        }
      }

      @media screen and (max-width: 1024px) {
        &:nth-child(2) {
          display: none;
        }
      }
    }

    th {
      color: var(--color-heading);
      font-size: 0.8rem;
      letter-spacing: 0.02rem;
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

  hr {
    background-color: var(--color-divider);
    border: none;
    color: var(--color-divider);
    height: 2px;
    margin: 0.5rem 0;
  }

  .new-version {
    display: grid;
    grid-template:
      'controls controls' auto
      'main main' auto
      'dependencies dependencies' auto
      'files files'
      'changelog changelog'
      / 5fr 4fr;
    column-gap: var(--spacing-card-md);

    @media screen and (min-width: 1024px) {
      grid-template:
        'controls controls' auto
        'main dependencies' auto
        'main files' 1fr
        'changelog changelog'
        / 5fr 4fr;
    }

    .controls {
      grid-area: controls;
      display: flex;
      flex-direction: row-reverse;
      gap: 0.5rem;
    }

    .main {
      grid-area: main;
    }

    .dependencies {
      grid-area: dependencies;

      .dependency-selector {
        .separator {
          margin: var(--spacing-card-sm) 0;
        }

        h4 {
          margin: 0 0 0.25rem 0;
        }

        .edit-info {
          display: flex;
          flex-wrap: wrap;
          row-gap: 0.25rem;

          .multiselect {
            max-width: 10rem;
          }

          .iconified-button {
            min-height: 2.5rem;
          }

          input,
          .multiselect,
          .iconified-button {
            margin: 0 0.5rem 0 0;
          }
        }
      }

      .new-dependencies {
        margin-top: var(--spacing-card-sm);
        display: flex;
        flex-wrap: wrap;

        .dependency {
          align-items: center;
          display: flex;
          flex-basis: 50%;
          margin-bottom: 0.5rem;

          .icon {
            width: 3rem;
            height: 3rem;
            margin-right: 0.5rem;
            border-radius: var(--size-rounded-xs);
            object-fit: contain;
          }

          .info {
            display: flex;
            flex-direction: column;
            justify-content: space-between;
            padding: 0.25rem;

            p {
              margin: 0;
            }

            .title {
              margin: 0 0.25rem 0 0;
            }
          }
        }
      }
    }

    .files {
      grid-area: files;

      .file-input {
        margin-top: 1rem;
      }

      .uploaded-files {
        .file {
          display: flex;
          align-items: center;
          margin-bottom: 0.25rem;
          flex-wrap: wrap;
          row-gap: 0.25rem;

          * {
            margin-left: 0.25rem;
          }
          .filename {
            margin: 0;
            font-weight: bold;
          }
        }
      }
    }

    .changelog {
      grid-area: changelog;
      display: flex;
      flex-direction: column;

      .separator {
        margin: var(--spacing-card-sm) 0;
      }

      .edit-wrapper {
        flex: 1;

        .textarea-wrapper {
          min-height: 10rem;
          height: 100%;
        }
      }
    }
  }
}

section.gallery {
  grid-area: gallery;

  .gallery-item {
    margin-top: 1rem;
    display: grid;
    grid-template:
      'info' auto
      'image' auto
      'buttons' auto
      / 1fr;

    @media screen and (min-width: 1024px) {
      grid-template:
        'info image' auto
        'buttons buttons'
        / 2fr 1fr;
    }

    column-gap: var(--spacing-card-md);
    row-gap: var(--spacing-card-sm);

    .info {
      grid-area: info;

      label {
        align-items: center;
        margin-top: var(--spacing-card-sm);

        span {
          flex: 1;
        }
      }
    }

    .image {
      grid-area: image;

      img {
        width: 100%;
        margin-top: 0.5rem;
        margin-bottom: 0.5rem;

        height: 14rem;
        object-fit: cover;

        border-radius: var(--size-rounded-card);
      }

      .bottom {
        display: flex;

        input,
        button {
          margin-right: 0.5rem;
        }
      }
    }

    .buttons {
      grid-area: buttons;

      hr {
        margin-top: 0.5rem;
      }
    }
  }
}

section.extra-links {
  grid-area: extra-links;

  label {
    align-items: center;
    margin-top: var(--spacing-card-sm);

    span {
      flex: 1;
    }
  }
}

section.license {
  grid-area: license;

  label {
    margin-top: var(--spacing-card-sm);
  }
}

section.donations {
  grid-area: donations;

  label {
    align-items: center;
    margin-top: var(--spacing-card-sm);

    span {
      flex: 1;
    }
  }

  button {
    margin: 0.5rem 0;
  }
}

.footer {
  grid-area: footer;
}

.choose-image {
  cursor: pointer;
}

.card {
  margin-bottom: 0;
}
</style>
