<template>
  <div class="page-contents">
    <header class="card">
      <div class="columns">
        <h3 class="column-grow-1">Edit project</h3>
        <nuxt-link
          :to="`/${project.project_type}/${
            project.slug ? project.slug : project.id
          }/settings`"
          class="iconified-button column"
        >
          <CrossIcon />
          Cancel
        </nuxt-link>
        <button
          v-if="
            project.status === 'rejected' ||
            project.status === 'draft' ||
            project.status === 'unlisted'
          "
          title="Submit for review"
          class="iconified-button column"
          :disabled="!$nuxt.$loading"
          @click="saveProjectReview"
        >
          <CheckIcon />
          Submit for review
        </button>
        <button
          title="Save"
          class="iconified-button brand-button-colors column"
          :disabled="!$nuxt.$loading"
          @click="saveProjectNotForReview"
        >
          <SaveIcon />
          Save changes
        </button>
      </div>
      <div v-if="showKnownErrors" class="known-errors">
        <ul>
          <li v-if="newProject.title === ''">Your project must have a name.</li>
          <li v-if="newProject.description === ''">
            Your project must have a summary.
          </li>
          <li v-if="newProject.slug === ''">
            Your project must have a vanity URL.
          </li>
          <li v-if="!savingAsDraft && newProject.body === ''">
            Your project must have a body to submit for review.
          </li>
          <li v-if="!savingAsDraft && project.versions.length < 1">
            Your project must have at least one version to submit for review.
          </li>
          <li
            v-if="
              license === null || license_url === null || license_url === ''
            "
          >
            Your project must have a license.
          </li>
        </ul>
      </div>
    </header>
    <section class="card essentials">
      <label>
        <span>
          <h3>Name<span class="required">*</span></h3>
          <span>
            Be creative! Generic project names will be harder to search for.
          </span>
        </span>
        <input
          v-model="newProject.title"
          :class="{ 'known-error': newProject.title === '' && showKnownErrors }"
          type="text"
          placeholder="Enter the name"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label>
        <span>
          <h3>Summary<span class="required">*</span></h3>
          <span>
            Give a short description of your project that will appear on search
            pages.
          </span>
        </span>
        <input
          v-model="newProject.description"
          :class="{
            'known-error': newProject.description === '' && showKnownErrors,
          }"
          type="text"
          placeholder="Enter the summary"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label>
        <span>
          <h3>Categories</h3>
          <span class="no-padding">
            Select up to 3 categories that will help others <br />
            find your project.
          </span>
        </span>
        <Multiselect
          id="categories"
          v-model="newProject.categories"
          :options="selectableCategories"
          :custom-label="
            (value) => value.charAt(0).toUpperCase() + value.slice(1)
          "
          :loading="$tag.categories.length === 0"
          :multiple="true"
          :searchable="true"
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
          @input="setCategories"
        />
      </label>
      <label>
        <span>
          <h3>Additional Categories</h3>
          <span class="no-padding">
            Select more categories that will help others <br />
            find your project. These are searchable, but not <br />
            displayed in search.
          </span>
        </span>
        <Multiselect
          id="additional_categories"
          v-model="newProject.additional_categories"
          :options="selectableAdditionalCategories"
          :custom-label="
            (value) => value.charAt(0).toUpperCase() + value.slice(1)
          "
          :loading="$tag.categories.length === 0"
          :multiple="true"
          :searchable="true"
          :show-no-results="false"
          :close-on-select="false"
          :clear-on-select="false"
          :show-labels="false"
          :max="255"
          :limit="6"
          :hide-selected="true"
          placeholder="Choose additional categories"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
          @input="setCategories"
        />
      </label>
      <label class="vertical-input">
        <span>
          <h3>Vanity URL (slug)<span class="required">*</span></h3>
          <span class="slug-description"
            >https://modrinth.com/{{ project.project_type.toLowerCase() }}/{{
              newProject.slug ? newProject.slug : 'your-slug'
            }}
          </span>
        </span>
        <input
          id="name"
          v-model="newProject.slug"
          :class="{ 'known-error': newProject.slug === '' && showKnownErrors }"
          type="text"
          placeholder="Enter the vanity URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="card project-icon">
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
      <SmartFileInput
        :max-size="262144"
        :show-icon="false"
        accept="image/png,image/jpeg,image/gif,image/webp"
        class="choose-image"
        prompt="Choose image or drag it here"
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
        @change="showPreviewImage"
      />
      <button
        class="iconified-button"
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
        @click="
          icon = null
          previewImage = null
          iconChanged = true
        "
      >
        <TrashIcon />
        Reset
      </button>
    </section>
    <section
      v-if="project.project_type !== 'resourcepack'"
      class="card game-sides"
    >
      <div class="columns">
        <div>
          <h3>Supported environments</h3>
          <span>
            Let others know what environments your project supports.
          </span>
        </div>
        <div class="labeled-control">
          <h3>Client<span class="required">*</span></h3>
          <Multiselect
            v-model="clientSideType"
            placeholder="Select one"
            :options="sideTypes"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
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
          <h3>Server<span class="required">*</span></h3>
          <Multiselect
            v-model="serverSideType"
            placeholder="Select one"
            :options="sideTypes"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
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
          Description<span class="required">*</span>
        </label>
      </h3>
      <span>
        You can type an extended description of your mod here. This editor
        supports
        <a
          class="text-link"
          href="https://guides.github.com/features/mastering-markdown/"
          target="_blank"
          rel="noopener noreferrer"
          >Markdown</a
        >. HTML can also be used inside your description, not including styles,
        scripts, and iframes (though YouTube iframes are allowed).
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
            :class="{
              'known-error': newProject.body === '' && showKnownErrors,
            }"
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
        <h3>License<span class="required">*</span></h3>
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
            placeholder="Choose license..."
            track-by="short"
            label="short"
            :options="$tag.licenses"
            :custom-label="(value) => value.short.toUpperCase()"
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
          Remove link
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

import CrossIcon from '~/assets/images/utils/x.svg?inline'
import CheckIcon from '~/assets/images/utils/check.svg?inline'
import PlusIcon from '~/assets/images/utils/plus.svg?inline'
import SaveIcon from '~/assets/images/utils/save.svg?inline'
import TrashIcon from '~/assets/images/utils/trash.svg?inline'

import ThisOrThat from '~/components/ui/ThisOrThat'
import SmartFileInput from '~/components/ui/SmartFileInput'

export default {
  components: {
    SmartFileInput,
    ThisOrThat,
    Multiselect,
    CrossIcon,
    CheckIcon,
    PlusIcon,
    SaveIcon,
    TrashIcon,
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

      selectableCategories: [],
      selectableAdditionalCategories: [],

      isProcessing: false,
      previewImage: null,
      compiledBody: '',

      icon: null,
      iconChanged: false,

      sideTypes: ['Required', 'Optional', 'Unsupported'],

      isEditing: true,
      bodyViewMode: 'source',

      showKnownErrors: false,
      savingAsDraft: false,
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

    this.setCategories()
  },
  watch: {
    license(newValue, oldValue) {
      if (newValue == null) {
        this.license_url = ''
        return
      }

      switch (newValue.short) {
        case 'custom':
          if (oldValue === null || oldValue.short !== '') {
            this.license_url = ''
          }
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
    this.DELETE_PROJECT = 1 << 7
  },
  methods: {
    setCategories() {
      this.selectableCategories = this.$tag.categories
        .filter(
          (x) =>
            x.project_type === this.project.actualProjectType &&
            !this.newProject.additional_categories.includes(x.name)
        )
        .map((it) => it.name)

      this.selectableAdditionalCategories = this.$tag.categories
        .filter(
          (x) =>
            x.project_type === this.project.actualProjectType &&
            !this.newProject.categories.includes(x.name)
        )
        .map((it) => it.name)
    },
    checkFields() {
      const reviewConditions =
        this.newProject.body !== '' && this.newProject.versions.length > 0
      if (
        this.newProject.name !== '' &&
        this.newProject.description !== '' &&
        this.newProject.slug !== '' &&
        this.license.short !== null &&
        this.license_url !== null &&
        this.license_url !== ''
      ) {
        if (this.savingAsDraft) {
          return true
        } else if (reviewConditions) {
          return true
        }
      }
      this.showKnownErrors = true
      return false
    },
    async saveProjectReview() {
      this.savingAsDraft = false
      if (this.checkFields()) {
        this.isProcessing = true
        await this.saveProject()
      }
    },
    async saveProjectNotForReview() {
      this.savingAsDraft = true
      if (this.checkFields()) {
        await this.saveProject()
      }
    },
    async saveProject() {
      this.$nuxt.$loading.start()

      try {
        const data = {
          title: this.newProject.title,
          description: this.newProject.description,
          body: this.newProject.body,
          categories: this.newProject.categories,
          additional_categories: this.newProject.additional_categories,
          issues_url: this.newProject.issues_url
            ? this.newProject.issues_url
            : null,
          source_url: this.newProject.source_url
            ? this.newProject.source_url
            : null,
          wiki_url: this.newProject.wiki_url ? this.newProject.wiki_url : null,
          license_url: this.license_url,
          discord_url: this.newProject.discord_url
            ? this.newProject.discord_url
            : null,
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
          this.$defaultHeaders()
        )

        if (this.iconChanged) {
          await this.$axios.patch(
            `project/${this.newProject.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$defaultHeaders()
          )
        }

        this.newProject.license = {
          id: this.newProject.license.short,
          url: this.newProject.license.url,
        }

        this.$emit('update:project', this.newProject)

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
    'header' auto
    'essentials' auto
    'project-icon' auto
    'game-sides' auto
    'description' auto
    'extra-links' auto
    'license' auto
    'donations' auto
    'footer' auto
    / 1fr;

  @media screen and (min-width: 1024px) {
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
  }
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
  label {
    margin-bottom: 0.5rem;
  }

  @media screen and (min-width: 1024px) {
    input {
      margin-left: 1.5rem;
    }
  }
}

section.project-icon {
  grid-area: project-icon;

  img {
    max-width: 100%;
    margin-bottom: 0.25rem;
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

    div {
      flex: 2;
    }

    .labeled-control {
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

.required {
  color: var(--color-badge-red-bg);
}

.vertical-input {
  flex-direction: column;
  justify-content: left;
  align-items: unset;
  gap: 0.5rem;

  input {
    margin-left: 0 !important;
  }
}
</style>
