<template>
  <div v-compiled-markdown="mod.body" v-highlightjs class="markdown-body"></div>
</template>

<script>
export default {
  auth: false,
  props: {
    mod: {
      type: Object,
      default() {
        return {}
      },
    },
  },
  created() {
    this.$emit('update:link-bar', [['Description', '']])
  },
}
</script>

<style lang="scss" scoped>
.markdown-body {
  padding: 1rem;
  margin-bottom: var(--spacing-card-md);
  background: var(--color-raised-bg);
  border-radius: var(--size-rounded-card);
}
</style>
