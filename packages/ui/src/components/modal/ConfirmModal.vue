<template>
  <NewModal ref="modal" :noblur="noblur" danger :on-hide="onHide">
    <template #title>
      <slot name="title">
        <span class="font-extrabold text-contrast text-lg">{{ title }}</span>
      </slot>
    </template>
    <div>
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
      <div class="flex gap-2 mt-6">
        <ButtonStyled color="red">
          <button :disabled="action_disabled" @click="proceed">
            <TrashIcon />
            {{ proceedLabel }}
          </button>
        </ButtonStyled>
        <ButtonStyled>
          <button @click="modal.hide()">
            <XIcon />
            Cancel
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup>
import { renderString } from '@modrinth/utils'
import { ref } from 'vue'
import { TrashIcon, XIcon } from '@modrinth/assets'
import NewModal from './NewModal.vue'
import ButtonStyled from '../base/ButtonStyled.vue'

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
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
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
