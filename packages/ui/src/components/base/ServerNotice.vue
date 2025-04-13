<template>
  <Admonition :type="NOTICE_TYPE[props.level]">
    <template #header>
      {{ formatMessage(heading) }}
    </template>
    <template #actions>
      <ButtonStyled v-if="dismissable" circular>
        <button
          v-tooltip="formatMessage(messages.dismiss)"
          @click="() => (preview ? {} : emit('dismiss'))"
        >
          <XIcon />
        </button>
      </ButtonStyled>
    </template>
    <div v-if="message" class="markdown-body" v-html="renderString(message)" />
  </Admonition>
</template>

<script setup lang="ts">
import { renderString } from '@modrinth/utils'
import { Admonition } from '../index'
import { XIcon } from '@modrinth/assets'
import { defineMessages, type MessageDescriptor, useVIntl } from '@vintl/vintl'
import { computed } from 'vue'
import ButtonStyled from './ButtonStyled.vue'

const { formatMessage } = useVIntl()
const emit = defineEmits<{
  (e: 'dismiss'): void
}>()

const props = withDefaults(
  defineProps<{
    level: string
    message: string
    dismissable: boolean
    preview?: boolean
  }>(),
  {
    preview: false,
  },
)

const messages = defineMessages({
  info: {
    id: 'servers.notice.heading.info',
    defaultMessage: 'Info',
  },
  attention: {
    id: 'servers.notice.heading.attention',
    defaultMessage: 'Attention',
  },
  dismiss: {
    id: 'servers.notice.dismiss',
    defaultMessage: 'Dismiss',
  },
})

const NOTICE_HEADINGS: Record<string, MessageDescriptor> = {
  info: messages.info,
  warn: messages.attention,
  critical: messages.attention,
}

const NOTICE_TYPE: Record<string, 'info' | 'warning' | 'critical'> = {
  info: 'info',
  warn: 'warning',
  critical: 'critical',
}

const heading = computed(() => NOTICE_HEADINGS[props.level] ?? messages.info)
</script>
