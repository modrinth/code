<template>
  <Card>
    <div class="markdown-body" v-html="renderHighlightedString(body)"/>
  </Card>
</template>

<script setup>
import {Card, renderHighlightedString} from 'omorphia'
</script>

<script>
export default {
  name: "Description",
  data() {
    return {
      body: "",
    };
  },
  async mounted() {
    const response = await fetch('https://api.modrinth.com/v2/project/' + this.$route.params.id)
    this.body = (await response.json()).body;
  }
}
</script>

<style scoped lang="scss">
.markdown-body {
  :deep(hr), :deep(h1), :deep(h2) {
    max-width: max(60rem, 90%);
  }

  :deep(ul) {
    margin-left: 2rem;
  }
}
</style>
