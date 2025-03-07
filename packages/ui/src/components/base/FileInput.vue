<template>
  <label
    :class="{ 'long-style': longStyle, 'too-large': hasError }"
    @drop.prevent="handleDrop"
    @dragover.prevent
  >
    <slot v-if="!isProcessingFile">
      <UploadIcon aria-hidden="true" />
    </slot>
    <SpinnerIcon v-else aria-hidden="true" class="animate-spin" />
    {{ hasError ? 'Error: Too large file' : prompt }}
    <input
      :accept="accept"
      :disabled="disabled"
      :multiple="multiple"
      type="file"
      @change="handleChange"
    />
  </label>
</template>

<script lang="ts" setup>
import { SpinnerIcon, UploadIcon } from '@modrinth/assets'
import { ref } from 'vue'

const props = withDefaults(
  defineProps<{
    prompt: string
    multiple?: boolean
    accept: string
    /**
     * The max file size in bytes. Defaults to 1MB.
     */
    maxSize?: number
    longStyle?: boolean
    disabled?: boolean
    callback: (files: File[]) => Promise<void>
  }>(),
  {
    maxSize: 1024 * 1024,
    multiple: false,
    longStyle: false,
    disabled: false,
  },
)

const files = ref<File[]>([])
const isProcessingFile = ref<boolean>(false)
const hasError = ref<boolean>(false)

async function addFiles(newFiles: File[]) {
  if (newFiles.length === 0) return

  files.value.push(...newFiles)

  // If only one file can be selected, replace the previous file with the new one
  if (!props.multiple) {
    if (files.value.length !== 0) {
      files.value = [files.value.at(-1)!]
    }
  }

  hasError.value = false
  files.value = files.value.filter((v) => {
    if (v.size > props.maxSize) {
      hasError.value = true
      return false
    }

    return true
  })

  isProcessingFile.value = true
  await props.callback(files.value)
  isProcessingFile.value = false
}

function handleDrop(e: DragEvent) {
  if (e.dataTransfer?.files) {
    addFiles(Array.from(e.dataTransfer.files))
  }
}

function handleChange(e: Event) {
  const target = e.target as HTMLInputElement
  if (target.files) {
    addFiles(Array.from(target.files))
  }
}
</script>

<style lang="scss" scoped>
label {
  flex-direction: unset;
  max-height: unset;

  svg {
    height: 1rem;
  }

  input {
    display: none;
  }

  &.long-style {
    display: flex;
    padding: 1.5rem 2rem;
    justify-content: center;
    align-items: center;
    grid-gap: 0.5rem;
    background-color: var(--color-button-bg);
    border-radius: var(--radius-sm);
    border: dashed 0.3rem var(--color-contrast);
    cursor: pointer;
    color: var(--color-contrast);

    &.too-large {
      border: dashed 0.3rem var(--color-red);
    }
  }
}
</style>
