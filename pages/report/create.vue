<template>
  <div class="page-container">
    <div class="page-contents">
      <header class="columns">
        <h3 class="column-grow-1">File a report</h3>
        <button
          title="Create"
          class="brand-button column"
          :disabled="!this.$nuxt.$loading"
          @click="createReport"
        >
          Create
        </button>
      </header>
      <section class="info">
        <h3>Item ID</h3>
        <label>
          <span>
            The ID of the item you are reporting. For example, the item ID of a
            mod would be its mod ID, found on the right side of that mod's page
            under "Project ID".
          </span>
          <input v-model="itemId" type="text" placeholder="Enter the Item ID" />
        </label>
        <h3>Item Type</h3>
        <label>
          <span> The type of the item that is being reported </span>
          <multiselect
            id="item-type"
            v-model="itemType"
            :options="['mod', 'version', 'user']"
            :multiple="false"
            :searchable="false"
            :show-no-results="false"
            :show-labels="false"
            placeholder="Choose item type"
          />
        </label>
        <h3>Report Type</h3>
        <label>
          <span>
            The type of report. This is the category that this report falls
            under.
          </span>
          <multiselect
            id="report-type"
            v-model="reportType"
            :options="reportTypes"
            :multiple="false"
            :searchable="false"
            :show-no-results="false"
            :show-labels="false"
            placeholder="Choose report type"
          />
        </label>
      </section>
      <section class="description">
        <h3>
          <label
            for="body"
            title="You can type the of the long form of your description here."
          >
            Body
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
    </div>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import axios from 'axios'

export default {
  components: {
    Multiselect,
  },
  fetch() {
    if (this.$route.query.id) this.itemId = this.$route.query.id
    if (this.$route.query.t) this.itemType = this.$route.query.t
  },
  async asyncData() {
    const reportTypes = (
      await axios.get(`https://api.modrinth.com/api/v1/tag/report_type`)
    ).data

    return {
      reportTypes,
    }
  },
  data() {
    return {
      itemId: '',
      itemType: '',
      reportType: '',
      body: '',

      reportTypes: ['aaaa'],
    }
  },
  methods: {
    async createReport() {
      this.$nuxt.$loading.start()

      try {
        const data = {
          report_type: this.reportType,
          item_id: this.itemId,
          item_type: this.itemType,
          body: this.body,
        }

        await axios.post(
          'https://api.modrinth.com/api/v1/report',
          data,
          this.$auth.headers
        )

        await this.$router.replace(`/${this.itemType}/${this.itemId}`)
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
    'info         info        info' auto
    'description  description description' auto
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

section {
  @extend %card;

  padding: var(--spacing-card-md) var(--spacing-card-lg);
}

section.info {
  grid-area: info;
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
</style>
