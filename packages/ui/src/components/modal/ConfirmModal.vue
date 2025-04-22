<template>
  <NewModal ref="modal" :noblur="noblur" :danger="danger" :on-hide="onHide">
    <template #title>
      <slot name="title">
        <span class="font-extrabold text-contrast text-lg">{{ title }}</span>
      </slot>
    </template>
    <div class="flex flex-col gap-4">
      <template v-if="description">
        <div
          v-if="markdown"
          class="markdown-body max-w-[35rem]"
          v-html="renderString(description)"
        />
        <p v-else class="max-w-[35rem] m-0">
          {{ description }}
        </p>
      </template>
      <slot />
      <label v-if="hasToType" for="confirmation">
        <span>
          To confirm you want to proceed, type
          <span class="italic font-bold">{{ confirmationText }}</span> below:
        </span>
      </label>
      <input
        v-if="hasToType"
        id="confirmation"
        v-model="confirmation_typed"
        type="text"
        placeholder="Type here..."
        class="max-w-[20rem]"
      />
      <div class="flex gap-2">
        <ButtonStyled :color="danger ? 'red' : 'brand'">
          <button :disabled="action_disabled" @click="proceed">
            <component :is="proceedIcon" />
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
import { ref, computed } from 'vue'
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
    default: undefined,
    required: false,
  },
  proceedIcon: {
    type: Object,
    default: TrashIcon,
  },
  proceedLabel: {
    type: String,
    default: 'Proceed',
  },
  noblur: {
    type: Boolean,
    default: false,
  },
  danger: {
    type: Boolean,
    default: true,
  },
  onHide: {
    type: Function,
    default() {
      return () => {}
    },
  },
  markdown: {
    type: Boolean,
    default: true,
  },
})

const emit = defineEmits(['proceed'])
const modal = ref(null)

const confirmation_typed = ref('')

const action_disabled = computed(
  () =>
    props.hasToType &&
    confirmation_typed.value.toLowerCase() !== props.confirmationText.toLowerCase(),
)

function proceed() {
  modal.value.hide()
  confirmation_typed.value = ''
  emit('proceed')
}

function show() {
  modal.value.show()
}

defineExpose({ show })
</script>
