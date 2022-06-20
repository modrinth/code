<template>
  <div class="columns">
    <label class="button" @drop.prevent="handleDrop" @dragover.prevent>
      <span>
        <UploadIcon v-if="showIcon" />
        {{ prompt }}
      </span>
      <input
        type="file"
        :multiple="multiple"
        :accept="accept"
        @change="handleChange"
      />
    </label>
  </div>
</template>

<script>
import { fileIsValid } from '~/plugins/fileUtils'
import UploadIcon from '~/assets/images/utils/upload.svg?inline'

export default {
  name: 'StatelessFileInput',
  components: {
    UploadIcon,
  },
  props: {
    prompt: {
      type: String,
      default: 'Select file',
    },
    multiple: {
      type: Boolean,
      default: false,
    },
    accept: {
      type: String,
      default: null,
    },
    /**
     * The max file size in bytes
     */
    maxSize: {
      type: Number,
      default: null,
    },
    showIcon: {
      type: Boolean,
      default: true,
    },
  },
  methods: {
    onChange(addedFiles) {
      this.$emit('change', addedFiles)
    },
    /**
     * @param {FileList} filesToAdd
     */
    addFiles(filesToAdd) {
      if (!filesToAdd) return

      const validationOptions = { maxSize: this.maxSize, alertOnInvalid: true }
      const validFiles = [...filesToAdd].filter((file) =>
        fileIsValid(file, validationOptions)
      )

      if (validFiles.length > 0) {
        this.onChange(this.multiple ? validFiles : [validFiles[0]])
      }
    },
    /**
     * @param {DragEvent} e
     */
    handleDrop(e) {
      this.addFiles(e.dataTransfer.files)
    },
    /**
     * @param {Event} e native file input event
     */
    handleChange(e) {
      this.addFiles(e.target.files)
    },
  },
}
</script>

<style lang="scss" scoped>
label {
  cursor: pointer;
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: var(--spacing-card-sm) var(--spacing-card-md);
  margin-bottom: var(--spacing-card-sm);
}

span {
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  gap: 0.5rem;
  border: 2px dashed var(--color-divider-dark);
  border-radius: var(--size-rounded-control);
  padding: var(--spacing-card-md) var(--spacing-card-lg);

  svg {
    height: 1.25rem;
  }
}

input {
  display: none;
}
</style>
