<template>
  <Modal ref="modal" :header="`Report ${itemType}`">
    <div class="modal-report legacy-label-styles">
      <div class="markdown-body">
        <p>
          Modding should be safe for everyone, so we take abuse and malicious intent seriously at
          Modrinth. We want to hear about harmful content on the site that violates our
          <nuxt-link to="/legal/terms"> ToS </nuxt-link> and
          <nuxt-link to="/legal/rules"> Rules </nuxt-link>. Rest assured, we’ll keep your
          identifying information private.
        </p>
        <p v-if="itemType === 'project' || itemType === 'version'">
          Please <strong>do not</strong> use this to report bugs with the project itself. This form
          is only for submitting a report to Modrinth staff. If the project has an Issues link or a
          Discord invite, consider reporting it there.
        </p>
      </div>
      <label class="report-label" for="report-type">
        <span>
          <strong>Reason</strong>
        </span>
      </label>
      <Multiselect
        id="report-type"
        v-model="reportType"
        :options="$tag.reportTypes"
        :custom-label="(value) => value.charAt(0).toUpperCase() + value.slice(1)"
        :multiple="false"
        :searchable="false"
        :show-no-results="false"
        :show-labels="false"
        placeholder="Choose report type"
      />
      <label class="report-label" for="additional-information">
        <strong>Additional information</strong>
        <span> Include links and images if possible. Markdown formatting is supported. </span>
      </label>
      <div class="textarea-wrapper">
        <Chips v-model="bodyViewType" class="separator" :items="['source', 'preview']" />
        <div v-if="bodyViewType === 'source'" class="textarea-wrapper">
          <textarea id="body" v-model="body" spellcheck="true" />
        </div>
        <div v-else class="preview" v-html="renderString(body)" />
      </div>
      <div class="button-group">
        <button class="iconified-button" @click="cancel">
          <CrossIcon />
          Cancel
        </button>
        <button class="iconified-button brand-button" @click="submitReport">
          <CheckIcon />
          Report
        </button>
      </div>
    </div>
  </Modal>
</template>

<script>
import { Multiselect } from 'vue-multiselect'
import CrossIcon from '~/assets/images/utils/x.svg'
import CheckIcon from '~/assets/images/utils/check.svg'
import Modal from '~/components/ui/Modal.vue'
import Chips from '~/components/ui/Chips.vue'
import { renderString } from '~/helpers/parse.js'

export default {
  components: {
    Chips,
    CrossIcon,
    CheckIcon,
    Modal,
    Multiselect,
  },
  props: {
    itemType: {
      type: String,
      default: '',
    },
    itemId: {
      type: String,
      default: '',
    },
  },
  data() {
    return {
      reportType: '',
      body: '',
      bodyViewType: 'source',
    }
  },
  methods: {
    renderString,
    cancel() {
      this.reportType = ''
      this.body = ''
      this.bodyViewType = 'source'

      this.$refs.modal.hide()
    },
    async submitReport() {
      startLoading()
      try {
        const data = {
          report_type: this.reportType,
          item_id: this.itemId,
          item_type: this.itemType,
          body: this.body,
        }
        await useBaseFetch('report', {
          method: 'POST',
          body: data,
          ...this.$defaultHeaders(),
        })

        this.$refs.modal.hide()
      } catch (err) {
        this.$notify({
          group: 'main',
          title: 'An error occurred',
          text: err.data.description,
          type: 'error',
        })
      }
      stopLoading()
    },
    show() {
      this.$refs.modal.show()
    },
  },
}
</script>

<style scoped lang="scss">
.modal-report {
  padding: var(--spacing-card-bg);
  display: flex;
  flex-direction: column;

  .markdown-body {
    margin-bottom: 1rem;
  }

  .multiselect {
    max-width: 20rem;
    margin-bottom: 1rem;
  }

  .report-label {
    display: flex;
    flex-direction: column;
    align-items: flex-start;
  }

  .button-group {
    margin-left: auto;
    margin-top: 1.5rem;
  }

  .textarea-wrapper {
    margin-top: 1rem;
    height: 12rem;

    textarea {
      // here due to a bug in safari
      max-height: 9rem;
    }

    .preview {
      overflow-y: auto;
    }
  }
}
</style>
