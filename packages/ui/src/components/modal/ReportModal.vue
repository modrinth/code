<template>
  <Modal ref="modal" :header="`Report ${itemType}`" :noblur="noblur">
    <div class="modal-report universal-labels">
      <div class="markdown-body">
        <p>
          Modding should be safe for everyone, so we take abuse and malicious intent seriously at
          Modrinth. We want to hear about harmful content on the site that violates our
          <router-link to="/legal/terms">ToS</router-link> and
          <router-link to="/legal/rules">Rules</router-link>. Rest assured, we'll keep your
          identifying information private.
        </p>
        <p v-if="itemType === 'project' || itemType === 'version'">
          Please <strong>do not</strong> use this to report bugs with the project itself. This form
          is only for submitting a report to Modrinth staff. If the project has an Issues link or a
          Discord invite, consider reporting it there.
        </p>
      </div>
      <label for="report-type">
        <span class="label__title">Reason</span>
      </label>
      <DropdownSelect
        id="report-type"
        v-model="reportType"
        name="report-type"
        :options="reportTypes"
        :display-name="capitalizeString"
        default-value="Choose report type"
        class="multiselect"
      />
      <label for="report-body">
        <span class="label__title">Additional information</span>
        <span class="label__description markdown-body">
          Please provide additional context about your report. Include links and images if possible.
          <strong>Empty reports will be closed.</strong> This editor supports
          <a href="https://docs.modrinth.com/markdown" target="_blank">Markdown formatting</a>.
        </span>
      </label>
      <Chips v-model="bodyViewType" class="separator" :items="['source', 'preview']" />
      <div class="text-input textarea-wrapper">
        <textarea v-if="bodyViewType === 'source'" id="body" v-model="body" spellcheck="true" />
        <div v-else class="preview" v-html="renderString(body)" />
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
import { capitalizeString, renderString } from '@modrinth/utils'
import { ref } from 'vue'
import { CheckIcon, XIcon } from '@modrinth/assets'
import Chips from '../base/Chips.vue'
import DropdownSelect from '../base/DropdownSelect.vue'
import Modal from './Modal.vue'

defineProps({
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

  .textarea-wrapper {
    height: 10rem;

    :first-child {
      max-height: 8rem;
      transform: translateY(1rem);
    }
  }

  .preview {
    overflow-y: auto;
  }
}
</style>
