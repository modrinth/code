<template>
  <input
    :id="id"
    type="checkbox"
    class="switch stylized-toggle"
    :checked="checked"
    @change="toggle"
  />
</template>

<script>
export default {
  props: {
    id: {
      type: String,
      required: true,
    },
    modelValue: {
      type: Boolean,
    },
    checked: {
      type: Boolean,
      required: true,
    },
  },
  emits: ['update:modelValue'],
  methods: {
    toggle() {
      if (!this.disabled) {
        this.$emit('update:modelValue', !this.modelValue)
      }
    },
  },
}
</script>
