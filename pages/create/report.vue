<template>
  <div class="page-container">
    <div class="page-contents">
      <header class="card columns">
        <h3 class="column-grow-1">File a report</h3>
        <button
          title="Create"
          class="brand-button-colors iconified-button column"
          :disabled="!$nuxt.$loading"
          @click="createReport"
        >
          <CheckIcon />
          Submit
        </button>
      </header>
      <section class="card info">
        <label>
          <span>
            <h3>Item ID</h3>
            <span>
              The ID of the item you are reporting. For example, the item ID of
              a project would be its project ID, found on the right side of that
              project's page under "Project ID".
            </span>
          </span>
          <input v-model="itemId" type="text" placeholder="Enter the item ID" />
        </label>
        <label>
          <span>
            <h3>Item type</h3>
            <span class="no-padding"
              >The type of the item that is being reported.</span
            >
          </span>
          <multiselect
            id="item-type"
            v-model="itemType"
            :options="['project', 'version', 'user']"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
            :multiple="false"
            :searchable="false"
            :show-no-results="false"
            :show-labels="false"
            placeholder="Choose item type"
          />
        </label>
        <label>
          <span>
            <h3>Report type</h3>
            <span class="no-padding">
              The type of report. This is the category that this report falls
              under.
            </span>
          </span>
          <multiselect
            id="report-type"
            v-model="reportType"
            :options="reportTypes"
            :custom-label="
              (value) => value.charAt(0).toUpperCase() + value.slice(1)
            "
            :multiple="false"
            :searchable="false"
            :show-no-results="false"
            :show-labels="false"
            placeholder="Choose report type"
          />
        </label>
      </section>
      <section class="card description">
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
          editor supports
          <a
            href="https://guides.github.com/features/mastering-markdown/"
            target="_blank"
            rel="noopener noreferrer"
            class="text-link"
            >Markdown</a
          >.
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
    </div>
  </div>
</template>

<script>
import Multiselect from 'vue-multiselect'
import ThisOrThat from '~/components/ui/ThisOrThat'

import CheckIcon from '~/assets/images/utils/check.svg?inline'

export default {
  components: {
    Multiselect,
    ThisOrThat,
    CheckIcon,
  },
  async asyncData(data) {
    const reportTypes = (await data.$axios.get(`tag/report_type`)).data

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

      bodyViewMode: 'source',

      reportTypes: ['aaaa'],
    }
  },
  fetch() {
    if (this.$route.query.id) this.itemId = this.$route.query.id
    if (this.$route.query.t) this.itemType = this.$route.query.t
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

        await this.$axios.post('report', data, this.$defaultHeaders())

        switch (this.itemType) {
          case 'version': {
            const version = (await this.$axios.get(`version/${this.itemId}`))
              .data
            const project = (
              await this.$axios.get(`project/${version.project_id}`)
            ).data
            await this.$router.replace(
              `/${project.project_type}/${project.slug || project.id}/version/${
                this.itemId
              }`
            )
            break
          }
          case 'project': {
            const project = (await this.$axios.get(`project/${this.itemId}`))
              .data
            await this.$router.replace(
              `/${project.project_type}/${project.slug || project.id}`
            )
            break
          }
          default:
            await this.$router.replace(`/${this.itemType}/${this.itemId}`)
        }
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

section.info {
  grid-area: info;
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

.card {
  margin-bottom: 0;
}

.card span {
  margin-bottom: 1rem;
}

label {
  align-items: center;
}
</style>
