<template>
  <Modal ref="modal" :header="`Report ${props.itemType}`" :noblur="noblur">
    <div class="modal-report">
      <div class="markdown-body">
        <p>
          Modding should be safe for everyone, so we take abuse and malicious intent seriously at
          Modrinth. We want to hear about harmful content on the site that violates our
          <router-link to="/legal/terms">ToS</router-link>
          and
          <router-link to="/legal/rules">Rules</router-link>
          . Rest assured, we’ll keep your identifying information private.
        </p>
        <p v-if="props.itemType === 'project' || props.itemType === 'version'">
          Please <strong>do not</strong> use this to report bugs with the project itself. This form
          is only for submitting a report to Modrinth staff. If the project has an Issues link or a
          Discord invite, consider reporting it there.
        </p>
      </div>
      <div>
        <label class="report-label" for="report-type">
          <span>
            <strong>Reason</strong>
          </span>
        </label>
        <DropdownSelect
          id="report-type"
          v-model="reportType"
          :options="props.reportTypes"
          default-value="Choose report type"
          class="multiselect"
        />
      </div>
      <label class="report-label" for="additional-information">
        <strong>Additional information</strong>
        <span> Include links and images if possible. Markdown formatting is supported. </span>
      </label>
      <div>
        <div v-if="bodyViewType === 'source'" class="text-input textarea-wrapper">
          <Chips v-model="bodyViewType" class="separator" :items="['source', 'preview']" />
          <textarea id="body" v-model="body" spellcheck="true" />
        </div>
        <div v-else class="preview" v-html="renderString(body)"></div>
      </div>
      <div class="input-group push-right">
        <Button @click="cancel">
          <XIcon />
          Cancel
        </Button>
        <Button color="primary" @click="submitReport">
          <CheckIcon />
          Report
        </Button>
      </div>
    </div>
  </Modal>
</template>
<script setup>
import { Modal, Chips, XIcon, CheckIcon, DropdownSelect } from '@/components'
import { renderString } from '@/helpers/parse.js'
import { ref } from 'vue'

const props = defineProps({
  itemType: {
    type: String,
    default: '',
  },
  itemId: {
    type: String,
    default: '',
  },
  reportTypes: {
    type: Array,
    default: () => [],
  },
  submitReport: {
    type: Function,
    default: () => {},
  },
  noblur: {
    type: Boolean,
    default: false,
  },
})

const reportType = ref('')
const body = ref('')
const bodyViewType = ref('source')

const modal = ref(null)

function cancel() {
  reportType.value = ''
  body.value = ''
  bodyViewType.value = 'source'

  modal.value.hide()
}

function show() {
  modal.value.show()
}

defineExpose({
  show,
})
</script>

<style scoped lang="scss">
.modal-report {
  padding: var(--gap-lg);
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.markdown-body {
  margin-bottom: 1rem;
}

.report-label {
  display: flex;
  flex-direction: column;
  align-items: flex-start;
  margin-bottom: 0.5rem;
}

.text-input {
  height: 12rem;
  gap: 1rem;

  textarea {
    // here due to a bug in safari
    max-height: 9rem;
  }

  .preview {
    overflow-y: auto;
  }
}
</style>
