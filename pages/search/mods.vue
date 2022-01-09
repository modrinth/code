<template>
  <div></div>
</template>

<script>
export default {
  name: 'Mods',
  asyncData(ctx) {
    ctx.params.projectType = 'mod'
  },
  head: {
    title: 'Mods - Modrinth',
    meta: [
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Mods',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Mods',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/mods`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped></style>
