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

<style lang="scss" scoped></style>
