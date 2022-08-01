<template>
  <div></div>
</template>

<script>
export default {
  name: 'ResourcePacks',
  asyncData(ctx) {
    ctx.params.projectType = 'resourcepack'
  },
  head: {
    title: 'Resource Packs - Modrinth',
    meta: [
      {
        hid: 'apple-mobile-web-app-title',
        name: 'apple-mobile-web-app-title',
        content: 'Resource Packs - Modrinth',
      },
      {
        hid: 'og:title',
        name: 'og:title',
        content: 'Resource Packs - Modrinth',
      },
      {
        hid: 'og:url',
        name: 'og:url',
        content: `https://modrinth.com/resourcepacks`,
      },
    ],
  },
}
</script>

<style lang="scss" scoped></style>
