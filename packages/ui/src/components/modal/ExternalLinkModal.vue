<template>
  <NewModal ref="modal" danger>
    <template #title>
      <span class="flex items-center gap-2 text-lg font-extrabold text-contrast">
        <ExternalIcon />
        {{ formatMessage(messages.title) }}
      </span>
    </template>

    <div class="flex flex-col gap-4 w-[450px]">
      <Admonition type="warning" header="Please be careful">
        {{ formatMessage(messages.warning) }}
      </Admonition>

      <div
        class="flex items-center gap-2 p-3 rounded-lg bg-bg text-contrast font-mono text-sm break-all"
      >
        <GlobeIcon class="flex-shrink-0 size-4" />
        {{ targetUrl }}
      </div>

      <div class="flex justify-end gap-2">
        <ButtonStyled>
          <button @click="cancel">
            <XIcon />
            {{ formatMessage(messages.cancel) }}
          </button>
        </ButtonStyled>
        <ButtonStyled color="brand">
          <button @click="proceed">
            <ExternalIcon />
            {{ formatMessage(messages.proceed) }}
          </button>
        </ButtonStyled>
      </div>
    </div>
  </NewModal>
</template>

<script setup>
import { ref } from 'vue'
import { defineMessages, useVIntl } from '@vintl/vintl'
import NewModal from './NewModal.vue'
import Admonition from '../base/Admonition.vue'
import ButtonStyled from '../base/ButtonStyled.vue'
import { ExternalIcon, XIcon, GlobeIcon } from '@modrinth/assets'

const { formatMessage } = useVIntl()

const messages = defineMessages({
  title: {
    id: 'modal.external_link.title',
    defaultMessage: 'Leaving Modrinth',
  },
  warning: {
    id: 'modal.external_link.warning',
    defaultMessage:
      'You are about to visit an external website. Please verify the link is from a trusted source before proceeding.',
  },
  proceed: {
    id: 'modal.external_link.proceed',
    defaultMessage: 'Continue to site',
  },
  cancel: {
    id: 'modal.external_link.cancel',
    defaultMessage: 'Cancel',
  },
})

const modal = ref(null)
const targetUrl = ref('')

const show = (url) => {
  targetUrl.value = url
  modal.value?.show()
  return new Promise((resolve) => {
    proceedCallback = resolve
  })
}

let proceedCallback = null

const cancel = () => {
  modal.value?.hide()
  if (proceedCallback) {
    proceedCallback(false)
    proceedCallback = null
  }
}

const proceed = () => {
  modal.value?.hide()
  if (proceedCallback) {
    proceedCallback(true)
    proceedCallback = null
  }
}

defineExpose({ show })
</script>

<style lang="scss" scoped>
.external-link-warning {
  display: flex;
  flex-direction: column;
  gap: var(--spacing-card-md);
  padding: var(--spacing-card-lg);

  .link-display {
    padding: var(--spacing-card-sm);
    background: var(--color-button-bg);
    border-radius: var(--size-rounded-sm);
    word-break: break-all;
    font-family: monospace;
  }

  .button-group {
    display: flex;
    justify-content: flex-end;
    gap: var(--spacing-card-sm);
  }
}
</style>
