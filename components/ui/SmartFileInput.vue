<template>
  <div class="columns">
    <label class="button" @drop.prevent="addFile" @dragover.prevent>
      <span>
        <UploadIcon />
        {{ prompt }}
      </span>
      <input
        type="file"
        :multiple="multiple"
        :accept="accept"
        @change="onChange"
      />
    </label>
  </div>
</template>

<script>
import UploadIcon from '~/assets/images/utils/upload.svg?inline'

export default {
  name: 'SmartFileInput',
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
  data() {
    return {
      files: [],
    }
  },
  methods: {
    onChange(files, shouldNotReset) {
      if (!shouldNotReset) this.files = files.target.files

      this.$emit('change', this.files)
    },
    addFile(e) {
      const droppedFiles = e.dataTransfer.files

      if (!this.multiple) this.files = []

      if (!droppedFiles) return
      ;[...droppedFiles].forEach((f) => {
        this.files.push(f)
      })

      if (!this.multiple && this.files.length > 0) this.files = [this.files[0]]

      if (this.files.length > 0) this.onChange(null, true)
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
