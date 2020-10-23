<template>
  <div>
    <slot></slot>
    <input
      :id="id"
      class="file-input"
      type="file"
      :accept="accept"
      :multiple="multiple"
      @change="onChange"
    />
    <label :for="id">{{ text }}</label>
  </div>
</template>

<script>
export default {
  name: 'FileInput',
  props: {
    defaultText: {
      type: String,
      default: '',
    },
    inputId: {
      type: String,
      default: '',
    },
    inputAccept: {
      type: String,
      default: '',
    },
    inputMultiple: {
      type: Boolean,
      default: true,
    },
  },
  data() {
    return {
      text: this.defaultText,
    }
  },
  methods: {
    onChange(files) {
      const length = files.target.length

      if (length === 0) {
        this.text = this.defaultText
      } else if (length === 1) {
        this.text = '1 file selected'
      } else if (length > 1) {
        this.text = length + ' files selected'
      }

      this.$emit('change', files)
    },
  },
}
</script>

<style lang="scss" scoped>
[type='file'] {
  border: 0;
  clip: rect(0, 0, 0, 0);
  height: 1px;
  overflow: hidden;
  padding: 0;
  position: absolute !important;
  white-space: nowrap;
  width: 1px;

  + label {
    cursor: pointer;
    border-radius: 5px;
    color: var(--color-grey-5);
    background-color: var(--color-grey-1);
    padding: 10px 20px;
  }

  &:focus + label,
  + label:hover,
  &:focus + label {
    background-color: var(--color-grey-2);
    color: var(--color-text);
  }
}
</style>
