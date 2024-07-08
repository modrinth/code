<template>
  <Modal ref="modal" :header="title" :noblur="noblur">
    <div class="modal-delete">
      <div class="markdown-body" v-html="renderString(description)" />
      <label v-if="hasToType" for="confirmation" class="confirmation-label">
        <span>
          <strong>To verify, type</strong>
          <em class="confirmation-text">{{ confirmationText }}</em>
          <strong>below:</strong>
        </span>
      </label>
      <div class="confirmation-input">
        <input
          v-if="hasToType"
          id="confirmation"
          v-model="confirmation_typed"
          type="text"
          placeholder="Type here..."
          @input="type"
        />
      </div>
      <div class="input-group push-right">
        <button class="btn" @click="modal.hide()">
          <XIcon />
          Cancel
        </button>
        <button class="btn btn-danger" :disabled="action_disabled" @click="proceed">
          <TrashIcon />
          {{ proceedLabel }}
        </button>
      </div>
    </div>
  </Modal>
</template>

<script setup>
import { renderString } from '@modrinth/utils'
import { ref } from 'vue'
import { TrashIcon, XIcon } from '@modrinth/assets'
import Modal from './Modal.vue'

const props = defineProps({
  confirmationText: {
    type: String,
    default: '',
  },
  hasToType: {
    type: Boolean,
    default: false,
  },
  title: {
    type: String,
    default: 'No title defined',
    required: true,
  },
  description: {
    type: String,
    default: 'No description defined',
    required: true,
  },
  proceedLabel: {
    type: String,
    default: 'Proceed',
  },
  noblur: {
    type: Boolean,
    default: false,
  },
})

const emit = defineEmits(['proceed'])
const modal = ref(null)

const action_disabled = ref(props.hasToType)
const confirmation_typed = ref('')

function proceed() {
  modal.value.hide()
  emit('proceed')
}

function type() {
  if (props.hasToType) {
    action_disabled.value =
      confirmation_typed.value.toLowerCase() !== props.confirmationText.toLowerCase()
  }
}

function show() {
  modal.value.show()
}

defineExpose({ show })
</script>

<style scoped lang="scss">
.modal-delete {
  padding: var(--gap-lg);
  display: flex;
  flex-direction: column;

  .markdown-body {
    margin-bottom: 1rem;
  }

  .confirmation-label {
    margin-bottom: 0.5rem;
  }

  .confirmation-text {
    padding-right: 0.25ch;
    margin: 0 0.25rem;
  }

  .confirmation-input {
    input {
      width: 20rem;
      max-width: 100%;
    }
  }

  .button-group {
    margin-left: auto;
    margin-top: 1.5rem;
  }
}
</style>
