<template>
  <div class="page-contents">
    <header class="card columns">
      <h3 class="column-grow-1">Edit project</h3>
      <nuxt-link
        :to="`/${project.project_type}/${
          project.slug ? project.slug : project.id
        }`"
        class="iconified-button column"
      >
        Back
      </nuxt-link>
      <button
        v-if="
          project.status === 'rejected' ||
          project.status === 'draft' ||
          project.status === 'unlisted'
        "
        title="Submit for approval"
        class="iconified-button column"
        :disabled="!$nuxt.$loading"
        @click="saveProjectReview"
      >
        Submit for approval
      </button>
      <button
        title="Save"
        class="iconified-button brand-button-colors column"
        :disabled="!$nuxt.$loading"
        @click="saveProject"
      >
        <CheckIcon />
        Save
      </button>
    </header>
    <section class="card essentials">
      <h3>Name</h3>
      <label>
        <span>
          Be creative! Generic project names will be harder to search for.
        </span>
        <input
          v-model="newProject.title"
          type="text"
          placeholder="Enter the name"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <h3>Summary</h3>
      <label>
        <span>
          Give a short description of your project that will appear on search
          pages.
        </span>
        <input
          v-model="newProject.description"
          type="text"
          placeholder="Enter the summary"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <h3>Categories</h3>
      <label>
        <span>
          Select up to 3 categories that will help others find your project.
        </span>
        <Multiselect
          id="categories"
          v-model="newProject.categories"
          :options="
            $tag.categories
              .filter((x) => x.project_type === project.project_type)
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
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <h3>Vanity URL (slug)</h3>
      <label>
        <span>
          Set this to something that will looks nice in your project's URL.
        </span>
        <input
          id="name"
          v-model="newProject.slug"
          type="text"
          placeholder="Enter the vanity URL slug"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="card project-icon rows">
      <h3>Icon</h3>
      <img
        :src="
          previewImage
            ? previewImage
            : newProject.icon_url && !iconChanged
            ? newProject.icon_url
            : 'https://cdn.modrinth.com/placeholder.svg'
        "
        alt="preview-image"
      />
      <file-input
        accept="image/png,image/jpeg,image/gif,image/webp"
        class="choose-image"
        prompt="Choose image or drag it here"
        @change="showPreviewImage"
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
      />
      <button
        class="iconified-button"
        @click="
          icon = null
          previewImage = null
          iconChanged = true
        "
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
      >
        <TrashIcon />
        Reset icon
      </button>
    </section>
    <section class="card game-sides">
      <h3>Supported environments</h3>
      <div class="columns">
        <span> Let others know what environments your project supports. </span>
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
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
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
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
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
          Description
        </label>
      </h3>
      <span>
        You can type an extended description of your mod here. This editor
        supports Markdown. Its syntax can be found
        <a
          href="https://guides.github.com/features/mastering-markdown/"
          target="_blank"
          rel="noopener noreferrer"
          class="text-link"
          >here</a
        >.
      </span>
      <ThisOrThat
        v-model="bodyViewMode"
        class="separator"
        :items="['source', 'preview']"
      />
      <div class="edit-wrapper">
        <div v-if="bodyViewMode === 'source'" class="textarea-wrapper">
          <textarea
            id="body"
            v-model="newProject.body"
            :disabled="(currentMember.permissions & EDIT_BODY) !== EDIT_BODY"
          />
        </div>
        <div
          v-if="bodyViewMode === 'preview'"
          v-highlightjs
          class="markdown-body"
          v-html="
            newProject.body
              ? $xss($md.render(newProject.body))
              : 'No body specified.'
          "
        ></div>
      </div>
    </section>
    <section class="card extra-links">
      <div class="title">
        <h3>External links</h3>
      </div>
      <label
        title="A place for users to report bugs, issues, and concerns about your project."
      >
        <span>Issue tracker</span>
        <input
          v-model="newProject.issues_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label
        title="A page/repository containing the source code for your project"
      >
        <span>Source code</span>
        <input
          v-model="newProject.source_url"
          type="url"
          placeholder="Enter a valid URL"
        />
      </label>
      <label
        title="A page containing information, documentation, and help for the project."
      >
        <span>Wiki page</span>
        <input
          v-model="newProject.wiki_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label title="An invitation link to your Discord server.">
        <span>Discord invite</span>
        <input
          v-model="newProject.discord_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="card license">
      <div class="title">
        <h3>License</h3>
      </div>
      <label>
        <span>
          It is very important to choose a proper license for your mod. You may
          choose one from our list or provide a URL to a custom license.
          <br />
          Confused? See our
          <a
            href="https://blog.modrinth.com/licensing-guide/"
            target="_blank"
            rel="noopener noreferrer"
            class="text-link"
          >
            licensing guide</a
          >
          for more information.
        </span>
        <div class="input-group">
          <Multiselect
            v-model="license"
            placeholder="Select one"
            track-by="short"
            label="name"
            :options="$tag.licenses"
            :searchable="true"
            :close-on-select="true"
            :show-labels="false"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          />
          <input
            v-model="license_url"
            type="url"
            placeholder="License URL"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          />
        </div>
      </label>
    </section>
    <section class="card donations">
      <div class="title">
        <h3>Donation links</h3>
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
</template>

<script>
import Multiselect from 'vue-multiselect'

import TrashIcon from '~/assets/images/utils/trash.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'

import FileInput from '~/components/ui/FileInput'
import ThisOrThat from '~/components/ui/ThisOrThat'

export default {
  components: {
    FileInput,
    ThisOrThat,
    Multiselect,
    TrashIcon,
    CheckIcon,
    PlusIcon,
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
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  data() {
    return {
      newProject: {},

      clientSideType: '',
      serverSideType: '',

      license: { short: '', name: '' },
      license_url: '',

      donationPlatforms: [],
      donationLinks: [],

      isProcessing: false,
      previewImage: null,
      compiledBody: '',

      icon: null,
      iconChanged: false,

      sideTypes: ['Required', 'Optional', 'Unsupported'],

      isEditing: true,
      bodyViewMode: 'source',
    }
  },
  fetch() {
    this.newProject = this.project

    this.newProject.license.short = this.newProject.license.id

    if (this.newProject.donation_urls) {
      for (const platform of this.newProject.donation_urls) {
        this.donationPlatforms.push({
          short: platform.id,
          name: platform.platform,
        })
        this.donationLinks.push(platform.url)
      }
    }

    this.license = {
      short: this.newProject.license.id,
      name: this.newProject.license.name,
    }

    this.license_url = this.newProject.license.url

    this.clientSideType =
      this.newProject.client_side.charAt(0) +
      this.newProject.client_side.slice(1)
    this.serverSideType =
      this.newProject.server_side.charAt(0) +
      this.newProject.server_side.slice(1)
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
  created() {
    this.UPLOAD_VERSION = 1 << 0
    this.DELETE_VERSION = 1 << 1
    this.EDIT_DETAILS = 1 << 2
    this.EDIT_BODY = 1 << 3
    this.MANAGE_INVITES = 1 << 4
    this.REMOVE_MEMBER = 1 << 5
    this.EDIT_MEMBER = 1 << 6
    this.DELETE_MOD = 1 << 7
  },
  methods: {
    async saveProjectReview() {
      this.isProcessing = true
      await this.saveProject()
    },
    async saveProject() {
      this.$nuxt.$loading.start()

      try {
        const data = {
          title: this.newProject.title,
          description: this.newProject.description,
          body: this.newProject.body,
          categories: this.newProject.categories,
          issues_url: this.newProject.issues_url,
          source_url: this.newProject.source_url,
          wiki_url: this.newProject.wiki_url,
          license_url: this.license_url,
          discord_url: this.newProject.discord_url,
          license_id: this.license.short,
          client_side: this.clientSideType.toLowerCase(),
          server_side: this.serverSideType.toLowerCase(),
          slug: this.newProject.slug,
          license: this.license.short,
          donation_urls: this.donationPlatforms.map((it, index) => {
            return {
              id: it.short,
              platform: it.name,
              url: this.donationLinks[index],
            }
          }),
        }

        if (this.isProcessing) {
          data.status = 'processing'
        }

        await this.$axios.patch(
          `project/${this.newProject.id}`,
          data,
          this.$auth.headers
        )

        if (this.iconChanged) {
          await this.$axios.patch(
            `project/${this.newProject.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$auth.headers
          )
        }

        this.$emit('update:featuredVersions', this.newProject)

        this.isEditing = false

        await this.$router.replace(
          `/${this.project.project_type}/${
            this.newProject.slug ? this.newProject.slug : this.newProject.id
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
    showPreviewImage(files) {
      const reader = new FileReader()
      this.iconChanged = true
      this.icon = files[0]
      reader.readAsDataURL(this.icon)

      reader.onload = (event) => {
        this.previewImage = event.target.result
      }
    },
  },
}
</script>

<style lang="scss" scoped>
.title {
  display: flex;
  justify-content: space-between;
  align-items: center;
  margin-top: 0.5rem;

  h3 {
    margin-top: 0;
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

.input-group {
  display: flex;
  flex-direction: column;

  * {
    margin-bottom: var(--spacing-card-sm);
  }
}

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

.page-contents {
  display: grid;
  grid-template:
    'header       header       header' auto
    'essentials   essentials   project-icon' auto
    'game-sides   game-sides   game-sides' auto
    'description  description  description' auto
    'extra-links  extra-links  extra-links' auto
    'license      license      license' auto
    'donations    donations    donations' auto
    'footer       footer       footer' auto
    / 4fr 1fr 2fr;
  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-md);
}

header {
  grid-area: header;
  padding: var(--spacing-card-md) var(--spacing-card-lg);

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
    width: 9rem;
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
