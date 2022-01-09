<template>
  <div class="columns">
    <label class="iconified-button" @drop.prevent="addFile" @dragover.prevent>
      <UploadIcon />
      {{ prompt }}
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
}

input {
  display: none;
}
</style>
