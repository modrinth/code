<template>
  <button class="code" :class="{ copied }" :title="formatMessage(copiedMessage)" @click="copyText">
    <span>{{ text }}</span>
    <CheckIcon v-if="copied" />
    <ClipboardCopyIcon v-else />
  </button>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useVIntl, defineMessage } from '@vintl/vintl'
import { CheckIcon, ClipboardCopyIcon } from '@modrinth/assets'

const copiedMessage = defineMessage({
  id: 'omorphia.component.copy.action.copy',
  defaultMessage: 'Copy code to clipboard',
})
const { formatMessage } = useVIntl()

const props = defineProps<{ text: string }>()

const copied = ref(false)

async function copyText() {
  await navigator.clipboard.writeText(props.text)
  copied.value = true
}
</script>

<style lang="scss" scoped>
.code {
  color: var(--color-text);
  display: inline-flex;
  grid-gap: 0.5rem;
  font-family: var(--mono-font);
  font-size: var(--font-size-sm);
  margin: 0;
  padding: 0.25rem 0.5rem;
  background-color: var(--color-button-bg);
  width: fit-content;
  border-radius: 10px;
  user-select: text;
  transition:
    opacity 0.5s ease-in-out,
    filter 0.2s ease-in-out,
    transform 0.05s ease-in-out,
    outline 0.2s ease-in-out;

  @media (prefers-reduced-motion) {
    transition: none !important;
  }

  svg {
    width: 1em;
    height: 1em;
  }

  &:hover {
    filter: brightness(0.85);
  }

  &:active {
    transform: scale(0.95);
    filter: brightness(0.8);
  }
}
</style>
