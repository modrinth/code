<template>
  <div class="columns">
    <label class="button" @drop.prevent="handleDrop" @dragover.prevent>
      <span>
        <UploadIcon />
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
  },
  methods: {
    onChange(addedFiles) {
      this.$emit('change', addedFiles)
    },
    addFiles(filesToAdd) {
      if (!filesToAdd) return

      if (!this.multiple && filesToAdd.length > 0) {
        this.onChange([filesToAdd[0]])
        return
      }

      this.onChange(filesToAdd)
    },
    handleDrop(e) {
      this.addFiles(e.dataTransfer.files)
    },
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
