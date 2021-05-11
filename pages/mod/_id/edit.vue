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
        v-if="mod.status === 'rejected' || mod.status === 'draft'"
        title="Submit for Review"
        class="button column"
        :disabled="!this.$nuxt.$loading"
        @click="saveModReview"
      >
        Submit for Review
      </button>
      <button
        title="Save"
        class="brand-button column"
        :disabled="!this.$nuxt.$loading"
        @click="saveMod"
      >
        Save
      </button>
    </header>
    <section class="essentials">
      <h3>Name</h3>
      <label>
        <span>
          Be creative. TechCraft v7 won't be searchable and won't be clicked on
        </span>
        <input v-model="mod.title" type="text" placeholder="Enter the name" />
      </label>
      <h3>Summary</h3>
      <label>
        <span>
          Give a quick description to your mod. It will appear in the search
        </span>
        <input
          v-model="mod.description"
          type="text"
          placeholder="Enter the summary"
        />
      </label>
      <h3>Categories</h3>
      <label>
        <span>
          Select up to 3 categories. They will help to find your mod
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
        />
      </label>
      <h3>Vanity URL (slug)</h3>
      <label>
        <span>
          Set this to something pretty, so URLs to your mod are more readable
        </span>
        <input
          id="name"
          v-model="mod.slug"
          type="text"
          placeholder="Enter the vanity URL's last bit"
        />
      </label>
    </section>
    <section class="mod-icon rows">
      <h3>Icon</h3>
      <div class="columns row-grow-1">
        <div class="column-grow-1 rows">
          <file-input
            accept="image/png,image/jpeg,image/gif,image/webp"
            class="choose-image"
            prompt="Choose image or drag it here"
            @change="showPreviewImage"
          />
          <ul class="row-grow-1">
            <li>Must be a square</li>
            <li>Minimum size is 100x100</li>
            <li>Acceptable formats are PNG, JPEG, GIF and WEBP</li>
          </ul>
          <button
            class="transparent-button"
            @click="
              icon = null
              previewImage = null
              iconChanged = true
            "
          >
            Reset icon
          </button>
        </div>
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
      </div>
    </section>
    <section class="game-sides">
      <h3>Supported environments</h3>
      <div class="columns">
        <span>
          Let others know if your mod is for clients, servers or universal. For
          example, IC2 will be required + required, while OptiFine will be
          required + no functionality
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
    <section class="description">
      <h3>
        <label
          for="body"
          title="You can type the of the long form of your description here."
        >
          Description
        </label>
      </h3>
      <span>
        You can type the of the long form of your description here. This editor
        supports markdown. You can find the syntax
        <a
          href="https://guides.github.com/features/mastering-markdown/"
          target="_blank"
          rel="noopener noreferrer"
          >here</a
        >.
      </span>
      <div class="columns">
        <div class="textarea-wrapper">
          <textarea id="body" v-model="mod.body"></textarea>
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
        />
      </label>
      <label title="A page/repository containing the source code">
        <span>Source code</span>
        <input
          v-model="mod.source_url"
          type="url"
          placeholder="Enter a valid URL"
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
        />
      </label>
      <label title="An inivitation link to your Discord server.">
        <span>Discord invite</span>
        <input
          v-model="mod.discord_url"
          type="url"
          placeholder="Enter a valid URL"
        />
      </label>
    </section>
    <section class="license">
      <div class="title">
        <h3>License</h3>
      </div>
      <label>
        <span>
          It is really important to choose a proper license for your mod. You
          may choose one from our list or provide a URL to your own license. URL
          field will be filled automatically for provided licenses
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
          />
          <input v-model="license_url" type="url" placeholder="License URL" />
        </div>
      </label>
    </section>
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
  </div>
</template>

<script>
import axios from 'axios'
import Multiselect from 'vue-multiselect'

import FileInput from '~/components/ui/FileInput'

export default {
  components: {
    FileInput,
    Multiselect,
  },
  async asyncData(data) {
    try {
      const [
        mod,
        availableCategories,
        availableLoaders,
        availableGameVersions,
        availableLicenses,
        availableDonationPlatforms,
      ] = (
        await Promise.all([
          axios.get(
            `https://api.modrinth.com/api/v1/mod/${data.params.id}`,
            data.$auth.headers
          ),
          axios.get(`https://api.modrinth.com/api/v1/tag/category`),
          axios.get(`https://api.modrinth.com/api/v1/tag/loader`),
          axios.get(`https://api.modrinth.com/api/v1/tag/game_version`),
          axios.get(`https://api.modrinth.com/api/v1/tag/license`),
          axios.get(`https://api.modrinth.com/api/v1/tag/donation_platform`),
        ])
      ).map((it) => it.data)

      mod.license = {
        short: mod.license.id,
        name: mod.license.name,
        url: mod.license.url,
      }

      if (mod.body_url && !mod.body) {
        mod.body = (await axios.get(mod.body_url)).data
      }

      const donationPlatforms = []
      const donationLinks = []

      if (mod.donation_urls) {
        for (const platform of mod.donation_urls) {
          donationPlatforms.push({
            short: platform.id,
            name: platform.platform,
          })
          donationLinks.push(platform.url)
        }
      }

      return {
        mod,
        clientSideType: mod.client_side.charAt(0) + mod.client_side.slice(1),
        serverSideType: mod.server_side.charAt(0) + mod.server_side.slice(1),
        availableCategories,
        availableLoaders,
        availableGameVersions,
        availableLicenses,
        license: {
          short: mod.license.id,
          name: mod.license.name,
        },
        license_url: mod.license.url,
        availableDonationPlatforms,
        donationPlatforms,
        donationLinks,
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

      sideTypes: ['Required', 'Optional', 'Unsupported'],
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
  created() {
    this.$emit('update:link-bar', [['Edit', 'edit']])
  },
  methods: {
    async saveModReview() {
      this.isProcessing = true
      await this.saveMod()
    },
    async saveMod() {
      this.$nuxt.$loading.start()

      try {
        const data = {
          title: this.mod.title,
          description: this.mod.description,
          body: this.mod.body,
          categories: this.mod.categories,
          issues_url: this.mod.issues_url,
          source_url: this.mod.source_url,
          wiki_url: this.mod.wiki_url,
          license_url: this.license_url,
          discord_url: this.mod.discord_url,
          license_id: this.license.short,
          client_side: this.clientSideType.toLowerCase(),
          server_side: this.serverSideType.toLowerCase(),
          slug: this.mod.slug,
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

        await axios.patch(
          `https://api.modrinth.com/api/v1/mod/${this.mod.id}`,
          data,
          this.$auth.headers
        )

        if (this.iconChanged) {
          await axios.patch(
            `https://api.modrinth.com/api/v1/mod/${this.mod.id}/icon?ext=${
              this.icon.type.split('/')[this.icon.type.split('/').length - 1]
            }`,
            this.icon,
            this.$auth.headers
          )
        }

        await this.$router.replace(
          `/mod/${this.mod.slug ? this.mod.slug : this.mod.id}`
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
    max-width: 50%;
    margin-left: var(--spacing-card-lg);
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
</style>
