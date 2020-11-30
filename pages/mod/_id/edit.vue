<template>
  <div class="page-container">
    <div class="page-contents">
      <header class="columns">
        <h2 class="column-grow-1">Edit Mod</h2>
        <button
          title="Save"
          class="brand-button column"
          :disabled="!this.$nuxt.$loading"
          @click="saveMod"
        >
          Save
        </button>
      </header>
      <EthicalAd class="advert" />
      <section class="essentials">
        <h3>Name</h3>
        <label>
          <span>
            Be creative. TechCraft v7 won't be searchable and won't be clicked
            on
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
              accept="image/png,image/jpeg,image/gif"
              class="choose-image"
              prompt="Choose image or drag it here"
              @change="showPreviewImage"
            />
            <ul class="row-grow-1">
              <li>Must be a square</li>
              <li>Minimum size is 100x100</li>
              <li>Acceptable formats are PNG, JPEG and GIF</li>
            </ul>
            <button
              class="transparent-button"
              @click="
                icon = null
                previewImage = null
              "
            >
              Reset icon
            </button>
          </div>
          <img
            :src="
              mod.icon_url
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
            Let others know if your mod is for clients, servers or universal.
            For example, IC2 will be required + required, while OptiFine will be
            required + no functionality
          </span>
          <div class="labeled-control">
            <h3>Client</h3>
            <Multiselect
              v-model="clientSideType"
              placeholder="Select one"
              track-by="id"
              label="label"
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
              track-by="id"
              label="label"
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
          You can type the of the long form of your description here. This
          editor supports markdown. You can find the syntax
          <a
            href="https://guides.github.com/features/mastering-markdown/"
            target="_blank"
            rel="noopener noreferrer"
            >here</a
          >.
        </span>
        <div class="columns">
          <div class="textarea-wrapper">
            <textarea id="body" v-model="body"></textarea>
          </div>
          <div v-compiled-markdown="body" class="markdown-body"></div>
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
            v-model="mod.wiki_url"
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
            may choose one from our list or provide a URL to your own license.
            URL field will be filled automatically for provided licenses
          </span>
          <div class="input-group">
            <Multiselect
              v-model="mod.license"
              placeholder="Select one"
              track-by="short"
              label="name"
              :options="availableLicenses"
              :searchable="true"
              :close-on-select="true"
              :show-labels="false"
            />
            <input
              v-model="mod.license.url"
              type="url"
              placeholder="License URL"
            />
          </div>
        </label>
      </section>
      <!--
      <section class="donations">
        <div class="title">
          <h3>Donation links</h3>
          <i>— this section is optional</i>
        </div>
      </section>
      -->
      <m-footer class="footer" centered />
    </div>
  </div>
</template>

<script>
import axios from 'axios'
import Multiselect from 'vue-multiselect'

import MFooter from '@/components/MFooter'
import FileInput from '@/components/FileInput'
import EthicalAd from '@/components/EthicalAd'

export default {
  components: {
    MFooter,
    FileInput,
    EthicalAd,
    Multiselect,
  },
  async asyncData(data) {
    const [
      mod,
      availableCategories,
      availableLoaders,
      availableGameVersions,
      availableLicenses,
      // availableDonationPlatforms,
    ] = (
      await Promise.all([
        axios.get(`https://api.modrinth.com/api/v1/mod/${data.params.id}`),
        axios.get(`https://api.modrinth.com/api/v1/tag/category`),
        axios.get(`https://api.modrinth.com/api/v1/tag/loader`),
        axios.get(`https://api.modrinth.com/api/v1/tag/game_version`),
        axios.get(`https://api.modrinth.com/api/v1/tag/license`),
        // axios.get(`https://api.modrinth.com/api/v1/tag/donation_platform`),
      ])
    ).map((it) => it.data)

    mod.license = {
      short: mod.license.id,
      name: mod.license.name,
    }

    const res = await axios.get(mod.body_url)

    return {
      mod,
      body: res.data,
      clientSideType: {
        label: mod.client_side,
        id: mod.client_side,
      },
      serverSideType: {
        label: mod.server_side,
        id: mod.server_side,
      },
      availableCategories,
      availableLoaders,
      availableGameVersions,
      availableLicenses,
      // availableDonationPlatforms,
    }
  },
  data() {
    return {
      previewImage: null,
      compiledBody: '',

      icon: null,

      sideTypes: [
        { label: 'Required', id: 'required' },
        { label: 'No functionality', id: 'no-functionality' },
        { label: 'Unsupported', id: 'unsupported' },
      ],
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
  methods: {
    async saveMod() {
      this.$nuxt.$loading.start()

      try {
        await axios.patch(
          `https://api.modrinth.com/api/v1/mod/${this.mod.id}`,
          {
            title: this.mod.title,
            description: this.mod.description,
            body: this.body,
            categories: this.mod.categories,
            issues_url: this.mod.issues_url,
            source_url: this.mod.source_url,
            wiki_url: this.mod.wiki_url,
            license_url: this.mod.license_url,
            discord_url: this.mod.discord_url,
            license_id: this.mod.license.short,
            client_side: this.clientSideType.id,
            server_side: this.serverSideType.id,
            slug: this.mod.mod_slug,
          },
          {
            headers: {
              Authorization: this.$auth.getToken('local'),
            },
          }
        )

        await this.$router.replace(`/mod/${this.mod.id}`)
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

    showPreviewImage(e) {
      const reader = new FileReader()
      this.icon = e.target.files[0]
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
    'essentials   essentials  mod-icon' auto
    'game-sides   game-sides  game-sides' auto
    'description  description description' auto
    'versions     versions    versions' auto
    'extra-links  license     license' auto
    'donations    donations   .' auto
    'footer       footer      footer' auto
    / 4fr 1fr 4fr;
  column-gap: var(--spacing-card-md);
  row-gap: var(--spacing-card-md);
}

header {
  @extend %card;

  grid-area: header;
  padding: var(--spacing-card-md) var(--spacing-card-lg);

  h2 {
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
}

.footer {
  grid-area: footer;
}

.choose-image {
  cursor: pointer;
}
</style>
