<template>
  <div class="columns">
    <label
      class="iconified-button"
      @drop.prevent="handleDrop"
      @dragover.prevent
    >
      <UploadIcon v-if="showIcon" />
      {{ prompt }}
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
  name: 'FileInput',
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
    shouldAlwaysReset: {
      type: Boolean,
      default: false,
    },
  },
  data() {
    return {
      files: [],
    }
  },
  methods: {
    addFiles(files, shouldNotReset) {
      if (!shouldNotReset || this.shouldAlwaysReset) this.files = files

      const validationOptions = { maxSize: this.maxSize, alertOnInvalid: true }
      this.files = [...this.files].filter((file) =>
        fileIsValid(file, validationOptions)
      )

      if (this.files.length > 0) {
        this.$emit('change', this.files)
      }
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
  flex-direction: unset;
  margin-bottom: 0;
  max-height: unset;

  svg {
    height: 1rem;
  }
}

input {
  display: none;
}

.known-error label {
  border-color: var(--color-badge-red-bg) !important;
  background-color: var(--color-warning-bg) !important;

  span {
    border-color: var(--color-badge-red-bg);
  }

  &::placeholder {
    color: var(--color-warning-text);
  }
}
</style>
