<template>
  <button class="code" :class="{ copied }" title="Copy code to clipboard" @click="copyText">
    <span>{{ text }}</span>
    <CheckIcon v-if="copied" />
    <ClipboardCopyIcon v-else />
  </button>
</template>

<script>
import CheckIcon from '~/assets/images/utils/check.svg?component'
import ClipboardCopyIcon from '~/assets/images/utils/clipboard-copy.svg?component'

export default {
  components: {
    CheckIcon,
    ClipboardCopyIcon,
  },
  props: {
    text: {
      type: String,
      required: true,
    },
  },
  data() {
    return {
      copied: false,
    }
  },
  methods: {
    async copyText() {
      await navigator.clipboard.writeText(this.text)
      this.copied = true
    },
  },
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
  background-color: var(--color-code-bg);
  width: min-content;
  border-radius: 10px;
  user-select: text;
  transition: opacity 0.5s ease-in-out, filter 0.2s ease-in-out, transform 0.05s ease-in-out,
    outline 0.2s ease-in-out;

  span {
    max-width: 10rem;
    overflow: hidden;
    text-overflow: ellipsis;
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
