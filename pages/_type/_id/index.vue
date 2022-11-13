<template>
  <div
    v-highlightjs
    class="markdown-body card"
    v-html="$xss($md.render(project.body))"
  ></div>
</template>

<script>
export default {
  auth: false,
  props: {
    project: {
      type: Object,
      default() {
        return {}
      },
    },
  },
}
</script>

<style lang="scss" scoped>
.markdown-body {
  max-width: calc(
    60rem - 2 * var(--spacing-card-lg) - 9px
  ); // $2.50 to anyone who can figure out why the 9px is needed
}
</style>
