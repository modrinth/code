<template>
  <div class="page-contents">
    <header class="columns">
      <h3 class="column-grow-1">Edit Mod</h3>
      <nuxt-link
        :to="'/mod/' + (mod.slug ? mod.slug : mod.id)"
        class="button column"
      >
        Back
      </nuxt-link>
      <button
        v-if="
          mod.status === 'rejected' ||
          mod.status === 'draft' ||
          mod.status === 'unlisted'
        "
        title="Submit for approval"
        class="button column"
        :disabled="!$nuxt.$loading"
        @click="saveModReview"
      >
        Submit for approval
      </button>
      <button
        title="Save"
        class="brand-button column"
        :disabled="!$nuxt.$loading"
        @click="saveMod"
      >
        Save
      </button>
    </header>
    <section class="essentials">
      <h3>Name</h3>
      <label>
        <span>
          Be creative. TechCraft v7 won't be searchable and won't be clicked on.
        </span>
        <input
          v-model="mod.title"
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
          Give a quick summary of your mod. This will appear in search.
        </span>
        <input
          v-model="mod.description"
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
          Select up to 3 categories. These will help others find your mod.
        </span>
        <multiselect
          id="categories"
          v-model="mod.categories"
          :options="availableCategories"
          :loading="availableCategories.length === 0"
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
          Set this to something pretty, so your mod's URL can be more readable.
        </span>
        <input
          id="name"
          v-model="mod.slug"
          type="text"
          placeholder="Enter the vanity URL's last bit"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="mod-icon rows">
      <h3>Icon</h3>
      <div class="columns row-grow-1">
        <div class="rows row-grow-1">
          <file-input
            accept="image/png,image/jpeg,image/gif,image/webp"
            class="choose-image"
            prompt="Choose an image or drag it here"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
            @change="showPreviewImage"
          />
          <ul class="row-grow-1">
            <li>Must be a square</li>
            <li>Minimum size is 100x100</li>
            <li>Acceptable formats are PNG, JPEG, GIF, and WEBP</li>
          </ul>
        </div>
        <div class="rows">
          <img
            :src="
              previewImage
                ? previewImage
                : mod.icon_url && !iconChanged
                ? mod.icon_url
                : 'https://cdn.modrinth.com/placeholder.svg'
            "
            alt="preview-image"
          />
          <button
            class="button"
            :disabled="
              (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
            "
            @click="
              icon = null
              previewImage = null
              iconChanged = true
            "
          >
            Reset icon
          </button>
        </div>
      </div>
    </section>
    <section class="game-sides">
      <h3>Supported environments</h3>
      <div class="columns">
        <span>
          Let others know if your mod is for clients, servers, or both. For
          example, Lithium would be optional for both sides, whereas Sodium
          would be required on the client and unsupported on the server.
        </span>
        <div class="labeled-control">
          <h3>Client</h3>
          <Multiselect
            v-model="mod.client_side"
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
            v-model="mod.server_side"
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
    <section class="description">
      <h3>
        <label
          for="body"
          title="You can type an extended description of your mod here."
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
          >here</a
        >.
      </span>
      <div class="columns">
        <div class="textarea-wrapper">
          <textarea
            id="body"
            v-model="mod.body"
            :disabled="(currentMember.permissions & EDIT_BODY) !== EDIT_BODY"
          ></textarea>
        </div>
        <div v-compiled-markdown="mod.body" class="markdown-body"></div>
      </div>
    </section>
    <section class="extra-links">
      <div class="title">
        <h3>External links</h3>
      </div>
      <label
        title="A place for users to report bugs, issues, and concerns about your mod."
      >
        <span>Issue tracker</span>
        <input
          v-model="mod.issues_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label title="A page/repository containing the source code for your mod.">
        <span>Source code</span>
        <input
          v-model="mod.source_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
      <label
        title="A page containing information, documentation, and help for the mod."
      >
        <span>Wiki page</span>
        <input
          v-model="mod.wiki_url"
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
          v-model="mod.discord_url"
          type="url"
          placeholder="Enter a valid URL"
          :disabled="
            (currentMember.permissions & EDIT_DETAILS) !== EDIT_DETAILS
          "
        />
      </label>
    </section>
    <section class="license">
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
            :options="availableLicenses"
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
    <!--
    <section class="donations">
      <div class="title">
        <h3>Donation links</h3>
        <button
          title="Add a link"
          class="button"
          :disabled="false"
          @click="
            donationPlatforms.push({})
            donationLinks.push('')
          "
        >
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
            :options="availableDonationPlatforms"
            :searchable="false"
            :close-on-select="true"
            :show-labels="false"
          />
        </label>
        <button
          class="button"
          @click="
            donationPlatforms.splice(index, 1)
            donationLinks.splice(index, 1)
          "
        >
          Remove Link
        </button>
        <hr />
      </div>
    </section>
    -->
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'

import getDifferences from '~/libs/getDifferences'

import FileInput from '~/components/ui/FileInput'

export default {
  components: {
    FileInput,
    Multiselect,
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
    currentMember: {
      type: Object,
      default() {
        return null
      },
    },
  },
  async asyncData(data) {
    try {
      const [
        savedMod,
        availableCategories,
        availableLoaders,
        availableGameVersions,
        availableLicenses,
        availableDonationPlatforms,
      ] = (
        await Promise.all([
          data.$axios.get(`mod/${data.params.id}`, data.$auth.headers),
          data.$axios.get(`tag/category`),
          data.$axios.get(`tag/loader`),
          data.$axios.get(`tag/game_version`),
          data.$axios.get(`tag/license`),
          data.$axios.get(`tag/donation_platform`),
        ])
      ).map((it) => it.data)

      savedMod.license = {
        short: savedMod.license.id,
        name: savedMod.license.name,
        url: savedMod.license.url,
      }

      if (savedMod.body_url && !savedMod.body) {
        savedMod.body = (await data.$axios.get(savedMod.body_url)).data
      }

      /*
      const donationPlatforms = []
      const donationLinks = []

      if (savedMod.donation_urls) {
        for (const platform of savedMod.donation_urls) {
          donationPlatforms.push({
            short: platform.id,
            name: platform.platform,
          })
          donationLinks.push(platform.url)
        }
      }
      */

      availableLicenses.sort((a, b) => a.name.localeCompare(b.name))
      return {
        savedMod,
        mod: { ...savedMod },
        availableCategories,
        availableLoaders,
        availableGameVersions,
        availableLicenses,
        license: {
          short: savedMod.license.id,
          name: savedMod.license.name,
        },
        license_url: savedMod.license.url,
        availableDonationPlatforms,
        // donationPlatforms,
        // donationLinks,
      }
    } catch {
      data.error({
        statusCode: 404,
        message: 'Mod not found',
      })
    }
  },
  data() {
    return {
      isProcessing: false,
      previewImage: null,
      compiledBody: '',

      icon: null,
      iconChanged: false,

      sideTypes: ['required', 'optional', 'unsupported'],

      isEditing: true,
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
  created() {
    this.$emit('update:link-bar', [['Edit', 'edit']])

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
    async saveModReview() {
      this.isProcessing = true
      await this.saveMod()
    },
    async saveMod() {
      this.$nuxt.$loading.start()

      const modChanges = getDifferences(this.savedMod, this.mod)

      try {
        const data = {
          ...({ title: modChanges.title } || {}),
          ...({ description: modChanges.description } || {}),
          ...({ body: modChanges.body } || {}),
          ...({ categories: modChanges.categories } || {}),
          ...({ issues_url: modChanges.issues_url } || {}),
          ...({ source_url: modChanges.source_url } || {}),
          ...({ wiki_url: modChanges.wiki_url } || {}),
          ...({ license_url: modChanges.license_url } || {}),
          ...({ discord_url: modChanges.discord_url } || {}),
          ...({ license_id: modChanges.license_id } || {}),
          ...({ client_side: modChanges.client_side } || {}),
          ...({ server_side: modChanges.server_side } || {}),
          ...({ slug: modChanges.slug } || {}),
          ...(modChanges.license
            ? { license: modChanges.license.short } || {}
            : {}),
          /*
          donation_urls: this.donationPlatforms.map((it, index) => {
            return {
              id: it.short,
              platform: it.name,
              url: this.donationLinks[index],
            }
          }),
        */
        }

        if (this.isProcessing) {
          data.status = 'processing'
        }

        await this.$axios.patch(`mod/${this.mod.id}`, data, this.$auth.headers)

        if (this.iconChanged) {
          await this.$axios.patch(
            `mod/${this.mod.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$auth.headers
          )
        }

        this.isEditing = false
        this.savedMod = this.mod

        this.$notify({
          group: 'main',
          title: 'Changes saved',
          type: 'success',
        })
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
  * {
    display: inline;
  }
  .button {
    margin-left: 1rem;
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
    'header       header      header' auto
    'advert       advert      advert' auto
    'essentials   essentials  essentials' auto
    'mod-icon     mod-icon    mod-icon' auto
    'game-sides   game-sides  game-sides' auto
    'description  description description' auto
    'versions     versions    versions' auto
    'extra-links  extra-links extra-links' auto
    'license      license     license' auto
    'donations    donations   donations' auto
    'footer       footer      footer' auto
    / 4fr 1fr 4fr;
  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-md);
}

header {
  @extend %card;

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

.advert {
  grid-area: advert;
}

section {
  @extend %card;

  padding: var(--spacing-card-md) var(--spacing-card-lg);
}

section.essentials {
  grid-area: essentials;
}

section.mod-icon {
  grid-area: mod-icon;

  img {
    align-self: flex-start;
    max-width: 6.08rem;
    margin-left: var(--spacing-card-lg);
    border-radius: var(--size-rounded-icon);
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

  & > .columns {
    align-items: stretch;
    min-height: 10rem;
    max-height: 40rem;

    & > * {
      flex: 1;
      max-width: 50%;
    }
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

a {
  text-decoration: underline;
  color: var(--color-link);
}
</style>
