<template>
  <div class="page-contents legacy-label-styles">
    <header class="header-card">
      <div class="header__row">
        <h2 class="header__title">Edit project</h2>
        <div class="input-group">
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
            class="iconified-button brand-button column"
            :disabled="!$nuxt.$loading"
            @click="saveProjectNotForReview"
          >
            <SaveIcon />
            Save changes
          </button>
        </div>
      </div>
      <div v-if="showKnownErrors" class="known-errors">
        <ul>
          <li v-if="newProject.title === ''">Your project must have a name.</li>
          <li v-if="newProject.description === ''">
            Your project must have a summary.
          </li>
          <li v-if="newProject.slug === ''">
            Your project cannot have an empty URL suffix.
          </li>
          <li v-if="!savingAsDraft && newProject.body === ''">
            Your project must have a body to submit for review.
          </li>
          <li v-if="!savingAsDraft && project.versions.length < 1">
            Your project must have at least one version to submit for review.
          </li>
          <li v-if="license.short === ''">Your project must have a license.</li>
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
          maxlength="64"
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
          maxlength="256"
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
      <div class="universal-labels">
        <label for="slug">
          <span class="label__title">URL<span class="required">*</span></span>
        </label>
        <div
          class="text-input-wrapper"
          :class="{ 'known-error': newProject.slug === '' && showKnownErrors }"
        >
          <div class="text-input-wrapper__before">
            https://modrinth.com/{{ project.project_type.toLowerCase() }}/
          </div>
          <!-- this is a textarea so it is horizontally scrollable on mobile -->
          <textarea
            id="slug"
            v-model="newProject.slug"
            type="text"
            maxlength="64"
            autocorrect="off"
            autocomplete="off"
            autocapitalize="none"
            rows="1"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
            @input="manualSlug = true"
          />
        </div>
      </div>
    </section>
    <section class="card project-icon">
      <h3>Icon</h3>
      <Avatar
        size="lg"
        class="avatar"
        :src="previewImage ? previewImage : newProject.icon_url"
        alt="preview-image"
      />
      <FileInput
        :max-size="262144"
        :show-icon="true"
        accept="image/png,image/jpeg,image/gif,image/webp"
        class="choose-image iconified-button"
        prompt="Choose image"
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
        @change="showPreviewImage"
      >
        <UploadIcon />
      </FileInput>
      <button
        class="iconified-button"
        :disabled="(currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS"
        @click="
          icon = null
          previewImage = null
          iconChanged = false
        "
      >
        <RevertIcon />
        Revert
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
      <Chips
        v-model="bodyViewMode"
        class="separator"
        :items="['source', 'preview']"
      />
      <div v-if="bodyViewMode === 'source'" class="resizable-textarea-wrapper">
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
          maxlength="2048"
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
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label
        title="A page containing information, documentation, and help for the project."
      >
        <span>Wiki page</span>
        <input
          v-model="newProject.wiki_url"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label
        class="no-margin"
        title="An invitation link to your Discord server."
      >
        <span>Discord invite</span>
        <input
          v-model="newProject.discord_url"
          type="url"
          maxlength="2048"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="universal-card license">
      <h3>License<span class="required">*</span></h3>
      <div class="adjacent-input">
        <label for="license-multiselect">
          <span class="label__description">
            It is very important to choose a proper license for your mod. You
            may choose one from our list or provide a custom license. You may
            also provide a custom URL to your chosen license; otherwise, the
            license text will be displayed.
            <span
              v-if="license && license.friendly === 'Custom'"
              class="label__subdescription"
            >
              Enter a valid
              <a
                href="https://spdx.org/licenses/"
                target="_blank"
                rel="noopener noreferrer"
                class="text-link"
                >SPDX license identifier</a
              >
              in the marked area. If your license does not have a SPDX
              identifier (for example, if you created the license yourself or if
              the license is Minecraft-specific), simply check the box and enter
              the name of the license instead.
            </span>
            <span class="label__subdescription">
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
          </span>
        </label>
        <div class="legacy-input-group">
          <Multiselect
            id="license-multiselect"
            v-model="license"
            placeholder="Select license..."
            track-by="short"
            label="friendly"
            :options="defaultLicenses"
            :searchable="true"
            :close-on-select="true"
            :show-labels="false"
            :class="{
              'known-error': license.short === '' && showKnownErrors,
            }"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          />
          <Checkbox
            v-if="license.requiresOnlyOrLater"
            v-model="allowOrLater"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          >
            Allow later editions of this license
          </Checkbox>
          <Checkbox
            v-if="license.friendly === 'Custom'"
            v-model="nonSpdxLicense"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          >
            License does not have a SPDX identifier
          </Checkbox>
          <input
            v-if="license.friendly === 'Custom'"
            v-model="license.short"
            type="text"
            maxlength="2048"
            :placeholder="nonSpdxLicense ? 'License name' : 'SPDX identifier'"
            :class="{
              'known-error': license.short === '' && showKnownErrors,
            }"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          />
          <input
            v-model="newProject.license.url"
            type="url"
            maxlength="2048"
            placeholder="License URL (optional)"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
          />
        </div>
      </div>
    </section>
    <section class="header-card donations">
      <div class="header__row">
        <h3 class="header__title">Donation links</h3>
        <div class="input-group">
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
      </div>
      <div v-for="(item, index) in donationPlatforms" :key="index">
        <label title="The donation link.">
          <span>Donation Link</span>
          <input
            v-model="donationLinks[index]"
            type="url"
            placeholder="Enter a valid URL"
            class="donation-link-input"
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
import RevertIcon from '~/assets/images/utils/undo.svg?inline'
import UploadIcon from '~/assets/images/utils/upload.svg?inline'

import Chips from '~/components/ui/Chips'
import FileInput from '~/components/ui/FileInput'
import Avatar from '~/components/ui/Avatar'
import Checkbox from '~/components/ui/Checkbox'

export default {
  components: {
    Checkbox,
    Avatar,
    FileInput,
    Chips,
    Multiselect,
    CrossIcon,
    CheckIcon,
    PlusIcon,
    SaveIcon,
    TrashIcon,
    RevertIcon,
    UploadIcon,
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

      defaultLicenses: [
        { friendly: 'Custom', short: '' },
        {
          friendly: 'All Rights Reserved/No License',
          short: 'All-Rights-Reserved',
        },
        { friendly: 'Apache License 2.0', short: 'Apache-2.0' },
        {
          friendly: 'BSD 2-Clause "Simplified" License',
          short: 'BSD-2-Clause',
        },
        {
          friendly: 'BSD 3-Clause "New" or "Revised" License',
          short: 'BSD-3-Clause',
        },
        {
          friendly: 'CC Zero (Public Domain equivalent)',
          short: 'CC0-1.0',
        },
        { friendly: 'CC-BY 4.0', short: 'CC-BY-4.0' },
        {
          friendly: 'CC-BY-SA 4.0',
          short: 'CC-BY-SA-4.0',
        },
        {
          friendly: 'CC-BY-NC 4.0',
          short: 'CC-BY-NC-4.0',
        },
        {
          friendly: 'CC-BY-NC-SA 4.0',
          short: 'CC-BY-NC-SA-4.0',
        },
        {
          friendly: 'CC-BY-ND 4.0',
          short: 'CC-BY-ND-4.0',
        },
        {
          friendly: 'CC-BY-NC-ND 4.0',
          short: 'CC-BY-NC-ND-4.0',
        },
        {
          friendly: 'GNU Affero General Public License v3',
          short: 'AGPL-3.0',
          requiresOnlyOrLater: true,
        },
        {
          friendly: 'GNU Lesser General Public License v2.1',
          short: 'LGPL-2.1',
          requiresOnlyOrLater: true,
        },
        {
          friendly: 'GNU Lesser General Public License v3',
          short: 'LGPL-3.0',
          requiresOnlyOrLater: true,
        },
        {
          friendly: 'GNU General Public License v2',
          short: 'GPL-2.0',
          requiresOnlyOrLater: true,
        },
        {
          friendly: 'GNU General Public License v3',
          short: 'GPL-3.0',
          requiresOnlyOrLater: true,
        },
        { friendly: 'ISC License', short: 'ISC' },
        { friendly: 'MIT License', short: 'MIT' },
        { friendly: 'Mozilla Public License 2.0', short: 'MPL-2.0' },
        { friendly: 'zlib License', short: 'Zlib' },
      ],
      license: { friendly: '', short: '', requiresOnlyOrLater: false },
      allowOrLater: false,
      nonSpdxLicense: false,

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
      manualSlug: false,
    }
  },
  fetch() {
    this.newProject = this.project

    if (this.newProject.donation_urls) {
      for (const platform of this.newProject.donation_urls) {
        this.donationPlatforms.push({
          short: platform.id,
          name: platform.platform,
        })
        this.donationLinks.push(platform.url)
      }
    }

    const licenseId = this.newProject.license.id
    const trimmedLicenseId = licenseId
      .replaceAll('-only', '')
      .replaceAll('-or-later', '')
      .replaceAll('LicenseRef-', '')
    this.license = this.defaultLicenses.find(
      (x) => x.short === trimmedLicenseId
    ) ?? { friendly: 'Custom', short: licenseId.replaceAll('LicenseRef-', '') }
    this.allowOrLater = licenseId.includes('-or-later')
    this.nonSpdxLicense = licenseId.includes('LicenseRef-')

    this.clientSideType =
      this.newProject.client_side.charAt(0) +
      this.newProject.client_side.slice(1)
    this.serverSideType =
      this.newProject.server_side.charAt(0) +
      this.newProject.server_side.slice(1)

    this.setCategories()
  },
  computed: {
    licenseId() {
      let id = ''

      if (this.nonSpdxLicense || this.license.short === 'All-Rights-Reserved')
        id += 'LicenseRef-'

      id += this.license.short

      if (this.license.requiresOnlyOrLater)
        id += this.allowOrLater ? 'or-later' : '-only'

      if (this.nonSpdxLicense) id.replaceAll(' ', '-')

      return id
    },
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
        this.newProject.title !== '' &&
        this.newProject.description !== '' &&
        this.newProject.slug !== '' &&
        this.license.short !== ''
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
          license_url: this.newProject.license.url
            ? this.newProject.license.url
            : null,
          discord_url: this.newProject.discord_url
            ? this.newProject.discord_url
            : null,
          license_id: this.licenseId,
          client_side: this.clientSideType.toLowerCase(),
          server_side: this.serverSideType.toLowerCase(),
          slug: this.newProject.slug,
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

        // While the emit below will take care of most changes,
        // some items require manually updating
        this.newProject.license.id = this.licenseId
        this.newProject.client_side = this.clientSideType.toLowerCase()
        this.newProject.server_side = this.serverSideType.toLowerCase()

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
  .checkbox,
  .legacy-input-group {
    flex: 3;
    height: fit-content;
  }
}

.legacy-input-group {
  display: flex;
  flex-direction: column;

  * {
    margin-bottom: var(--spacing-card-sm);
  }

  .multiselect {
    width: unset;
    height: inherit;
  }
}

.resizable-textarea-wrapper textarea {
  min-height: 20rem;
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
}

section.essentials {
  grid-area: essentials;

  @media screen and (min-width: 1024px) {
    input {
      margin-left: 1.5rem;
    }
  }
}

section.project-icon {
  grid-area: project-icon;
  display: flex;
  flex-direction: column;
  align-items: center;
  gap: var(--spacing-card-sm);

  .avatar {
    margin-bottom: var(--spacing-card-sm);
  }

  .iconified-button {
    margin-top: var(--spacing-card-sm);
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
    }
  }
}

section.description {
  grid-area: description;

  .separator {
    margin: var(--spacing-card-sm) 0;
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

.card,
.universal-card,
.header-card {
  margin-bottom: 0;
}

.required {
  color: var(--color-special-red);
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

.text-input-wrapper {
  width: 100%;
  display: flex;
  align-items: center;

  textarea {
    width: 100%;
    height: 100%;
    margin-left: 0 !important;
    white-space: nowrap;
    overflow-x: auto;
    overflow-y: none;
    resize: none;
    min-height: 0;
  }
  margin-bottom: var(--spacing-card-md);
}

.donation-link-input {
  width: 100%;
}
</style>
