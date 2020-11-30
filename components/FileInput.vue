<template>
  <label class="button">
    <span>
      {{ prompt }}
    </span>
    <input
      type="file"
      :multiple="multiple"
      :accept="accept"
      @change="(files) => $emit('change', files)"
    />
  </label>
</template>

<script>
export default {
  name: 'FileInput',
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
}
</script>

<style lang="scss" scoped>
label {
  display: flex;
  align-items: center;
  justify-content: center;
  text-align: center;
  padding: var(--spacing-card-sm) var(--spacing-card-md);
}

span {
  border: 2px dashed var(--color-divider-dark);
  border-radius: var(--size-rounded-control);
  padding: var(--spacing-card-md) var(--spacing-card-lg);
}

input {
  display: none;
}
</style>
